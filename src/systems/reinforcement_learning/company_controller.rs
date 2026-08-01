use crate::components::common::{Name, TimeToLive};
use crate::components::economy::money::{LastTickMoney, Money};
use crate::components::economy::offer::{Offer, OfferBundle};
use crate::components::economy::order::{Order, OrderBundle};
use crate::components::economy::processor::{Processor, Processors, Productive};
use crate::components::economy::production_speed::ProductionSpeed;
use crate::components::economy::recipe::Recipe;
use crate::components::economy::stock::Stock;
use crate::components::reinforcement_learning::action::CompanyAction;
use crate::components::reinforcement_learning::company_state::CompanyState;
use crate::components::reinforcement_learning::confidence::CompanyConfidence;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::processor::ProcessorPrice;
use crate::resources::economy::recipes::Recipes;
use crate::resources::economy::resources::Resources;
use crate::resources::reinforcement_learning::action_space::{ActionSpace, CompanyActionEnum};
use crate::resources::reinforcement_learning::backend::MyAutodiffBackend;
use crate::resources::reinforcement_learning::q_networks::{CompanyQState, QNetworkStore};
use bevy::prelude::*;
use burn::Tensor;
use itertools::Itertools;
use rand::Rng;

const LEARNING_RATE: f64 = 1e-3;
// Discount factor: how much future rewards are worth relative to immediate ones
const GAMMA: f64 = 0.95;
// Probability of picking a random action instead of the greedy one (exploration)
const EPSILON: f64 = 0.1;

pub fn control_companies(
    mut commands: Commands,
    mut q_networks: NonSendMut<QNetworkStore>,
    mut companies: Query<(
        Entity,
        &Name,
        &Stock,
        &mut Processors,
        &mut Money,
        &mut LastTickMoney,
        &mut CompanyState,
        &mut CompanyAction,
        &mut CompanyConfidence,
    )>,
    resources: Res<Resources>,
    recipes: Res<Recipes>,
    processor_price: Res<ProcessorPrice>,
    action_space: Res<ActionSpace>,
    marketplace: Res<Marketplace>,
) {
    let device = burn::backend::wgpu::WgpuDevice::default();
    let state_size =
        CompanyState::new(resources.resources.len(), recipes.recipes.len())
            .get_size(resources.resources.len(), recipes.recipes.len());
    let action_size = action_space.actions.len();
    // Sorted so action indices (BuyResource(i)) map to the same resource as state indices
    let sorted_resource_ids: Vec<usize> =
        resources.resources.keys().copied().sorted().collect();
    let mut rng = rand::rng();

    for (entity, _name, stock, mut processors, mut money, mut last_tick_money, mut last_state, mut last_action, mut confidence) in
        companies.iter_mut()
    {
        // Each company gets its own network and optimizer, initialized lazily
        let q_state = q_networks
            .0
            .entry(entity)
            .or_insert_with(|| CompanyQState::new(&device, state_size, action_size));

        // Build s': current observation after the environment responded to last action.
        // Keys are sorted so the tensor layout is deterministic across ticks.
        let mut processor_counts = vec![];
        for recipe_id in recipes.recipes.iter().map(|x| x.0).sorted() {
            let count = processors
                .processors
                .iter()
                .filter(|p| p.recipe.0 == *recipe_id)
                .count();
            processor_counts.push(count as f64);
        }
        // Always iterate over the full resource list (not marketplace keys) so the
        // vectors are always the same length the network was built for.
        let mut order_index = vec![];
        for &resource_id in &sorted_resource_ids {
            match marketplace.order_index.get(&resource_id).and_then(|v| v.as_ref()) {
                Some(o) => order_index.push(o.1),
                None => order_index.push(0.0),
            }
        }
        let mut price_index = vec![];
        for &resource_id in &sorted_resource_ids {
            match marketplace.price_index.get(&resource_id).and_then(|v| v.as_ref()) {
                Some(p) => price_index.push(p.1),
                None => price_index.push(0.0),
            }
        }
        let mut stock_vec = vec![];
        for resource in stock.resources.iter().map(|x| *x.0).sorted() {
            stock_vec.push(*stock.resources.get(&resource).unwrap());
        }
        let current_state = CompanyState {
            money: money.0,
            order_index,
            price_index,
            processor_counts,
            stock: stock_vec,
        };

        // --- Bellman / TD update ---
        // We observed transition (s, a, r, s'). Update only Q(s,a); leave all other
        // action slots unchanged so we don't regress on actions we didn't take this step.
        let s: Tensor<MyAutodiffBackend, 1> = last_state.as_tensor();
        let s_prime: Tensor<MyAutodiffBackend, 1> = current_state.as_tensor();
        let a = last_action.0;
        let reward = money.0 - last_tick_money.0;

        let current_q_data = q_state.forward(s.clone()).to_data();
        let current_q_slice = current_q_data.as_slice::<f32>().unwrap();

        let max_q_next = q_state
            .forward(s_prime.clone())
            .max()
            .to_data()
            .as_slice::<f32>()
            .unwrap()[0] as f64;

        let q_s_a = current_q_slice[a] as f64;
        // Q(s,a) ← Q(s,a) + α · (r + γ · max_a' Q(s',a') − Q(s,a))
        let new_q = q_s_a + LEARNING_RATE * (reward + GAMMA * max_q_next - q_s_a);

        let mut target_q: Vec<f32> = current_q_slice.to_vec();
        target_q[a] = new_q as f32;

        let target_tensor: Tensor<MyAutodiffBackend, 1> =
            Tensor::from_data(target_q.as_slice(), &device);

        q_state.train(s, target_tensor, LEARNING_RATE);

        // Compute Q-values for s' once; used for both action selection and confidence.
        let q_prime_data = q_state.forward(s_prime).to_data();
        let q_prime = q_prime_data.as_slice::<f32>().unwrap();

        // Epsilon-greedy: random action with probability EPSILON, greedy otherwise.
        let exploring = rng.random::<f64>() < EPSILON;
        let next_action_index = if exploring {
            rng.random_range(0..action_size)
        } else {
            q_prime
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0)
        };

        // Softmax confidence for greedy actions; None signals exploration.
        confidence.0 = if exploring {
            None
        } else {
            let max_q = q_prime.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_vals: Vec<f32> = q_prime.iter().map(|&q| (q - max_q).exp()).collect();
            let sum_exp: f32 = exp_vals.iter().sum();
            Some(exp_vals[next_action_index] / sum_exp)
        };

        let next_action = match action_space.actions.get(next_action_index) {
            Some(a) => a,
            None => continue,
        };

        match next_action {
            CompanyActionEnum::Nothing => {}
            CompanyActionEnum::BuyProcessor(recipe) => {
                if *recipe < recipes.recipes.len() && money.0 >= processor_price.0 {
                    money.0 -= processor_price.0;
                    processors.processors.push(Processor {
                        production_speed: ProductionSpeed(1.0),
                        productive: Productive(true),
                        recipe: Recipe(*recipe),
                    });
                }
            }
            CompanyActionEnum::SellProcessor(recipe) => {
                if *recipe < recipes.recipes.len() {
                    if let Some(i) = processors
                        .processors
                        .iter()
                        .position(|p| p.recipe.0 == *recipe)
                    {
                        processors.processors.remove(i);
                    }
                }
            }
            CompanyActionEnum::BuyResource(resource_idx, amount) => {
                if let Some(&resource_id) = sorted_resource_ids.get(*resource_idx) {
                    if let Some(Some((_, best_price))) =
                        marketplace.price_index.get(&resource_id)
                    {
                        commands.spawn(OrderBundle {
                            order: Order {
                                amount: *amount as f64,
                                max_price_per_unit: *best_price,
                                company: Some(entity),
                                resource: resource_id,
                            },
                            time_to_live: TimeToLive(100),
                        });
                    }
                }
            }
            CompanyActionEnum::SellResource(resource_idx, amount) => {
                if let Some(&resource_id) = sorted_resource_ids.get(*resource_idx) {
                    if let Some(Some((_, best_price))) =
                        marketplace.order_index.get(&resource_id)
                    {
                        commands.spawn(OfferBundle {
                            offer: Offer {
                                amount: *amount as f64,
                                price_per_unit: *best_price,
                                company: Some(entity),
                                resource: resource_id,
                            },
                            time_to_live: TimeToLive(100),
                        });
                    }
                }
            }
        }

        // Record (s', a') so the next tick can compute the transition (s', a', r', s'')
        *last_tick_money = LastTickMoney(money.0);
        *last_state = current_state;
        *last_action = CompanyAction(next_action_index);
    }
}
