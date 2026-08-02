use crate::components::common::{Name, TimeToLive};
use crate::components::economy::money::{LastTickMoney, Money};
use crate::components::economy::offer::{Offer, OfferBundle};
use crate::components::economy::order::{Order, OrderBundle};
use crate::components::economy::processor::{Processor, Processors, Productive};
use crate::components::economy::production_speed::ProductionSpeed;
use crate::components::economy::recipe::Recipe;
use crate::components::economy::stock::Stock;
use crate::components::reinforcement_learning::action::CompanyAction;
use crate::components::reinforcement_learning::company_state::{CompanyState, MONEY_SCALE};
use crate::components::reinforcement_learning::confidence::CompanyConfidence;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::processor::ProcessorPrice;
use crate::resources::economy::recipes::Recipes;
use crate::resources::economy::resources::Resources;
use crate::resources::reinforcement_learning::action_space::{ActionSpace, CompanyActionEnum};
use crate::resources::reinforcement_learning::backend::{MyAutodiffBackend, MyBackend};
use crate::resources::reinforcement_learning::q_networks::{CompanyQState, QNetworkStore, Transition};
use crate::resources::reinforcement_learning::training_history::TrainingHistory;
use bevy::prelude::*;
use burn::Tensor;
use itertools::Itertools;
use rand::Rng;

const LEARNING_RATE: f64 = 1e-3;
// Discount factor: how much future rewards are worth relative to immediate ones
const GAMMA: f64 = 0.95;
// Minibatch size sampled from the replay buffer each gradient step
const BATCH_SIZE: usize = 32;
// Exploration schedule: start almost fully random, anneal toward mostly-greedy so early
// episodes explore broadly and later ones exploit what was learned.
const EPS_START: f64 = 1.0;
const EPS_END: f64 = 0.05;
const EPS_DECAY_STEPS: f64 = 20_000.0;

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
    mut marketplace: ResMut<Marketplace>,
    mut training: ResMut<TrainingHistory>,
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
        for &resource_id in &sorted_resource_ids {
            stock_vec.push(stock.resources.get(&resource_id).copied().unwrap_or(0.0));
        }
        let current_state = CompanyState {
            money: money.0,
            order_index,
            price_index,
            processor_counts,
            stock: stock_vec,
        };

        // --- Experience replay ---
        // We observed transition (s, a, r, s'); store it and train on a decorrelated
        // minibatch rather than only this single, highly-correlated sample.
        let a = last_action.0;
        let raw_reward = money.0 - last_tick_money.0;
        // Scale the reward like the money input so TD targets stay in the same range as
        // the network's outputs; otherwise Huber loss clips every error into its linear
        // region and gradients stall.
        let reward = (raw_reward / MONEY_SCALE) as f32;

        q_state.remember(Transition {
            state: last_state.as_vec(),
            action: a,
            reward,
            next_state: current_state.as_vec(),
        });

        if let Some(batch) = q_state.sample_batch(BATCH_SIZE, &mut rng) {
            let bs = batch.len();
            let mut s_flat: Vec<f32> = Vec::with_capacity(bs * state_size);
            let mut sp_flat: Vec<f32> = Vec::with_capacity(bs * state_size);
            for t in &batch {
                s_flat.extend_from_slice(&t.state);
                sp_flat.extend_from_slice(&t.next_state);
            }

            // Baseline the target on the online net's current Q(s) so untaken actions
            // produce zero loss; then overwrite each taken action with its Bellman target
            // r + γ·max_a' Q_target(s',a'). The gradient step does the actual moving — we
            // must NOT also scale by the learning rate here (that shrinks the effective
            // step to ~α² and stalls learning).
            let s_inner: Tensor<MyBackend, 2> =
                Tensor::<MyBackend, 1>::from_data(s_flat.as_slice(), &device)
                    .reshape([bs, state_size]);
            let sp_inner: Tensor<MyBackend, 2> =
                Tensor::<MyBackend, 1>::from_data(sp_flat.as_slice(), &device)
                    .reshape([bs, state_size]);

            let online_q = q_state.infer_batch(s_inner).to_data();
            let mut target_flat: Vec<f32> = online_q.as_slice::<f32>().unwrap().to_vec();
            let max_next_data = q_state.target_forward_batch(sp_inner).max_dim(1).to_data();
            let max_next = max_next_data.as_slice::<f32>().unwrap();

            for (i, t) in batch.iter().enumerate() {
                target_flat[i * action_size + t.action] = t.reward + GAMMA as f32 * max_next[i];
            }

            let input: Tensor<MyAutodiffBackend, 2> =
                Tensor::<MyAutodiffBackend, 1>::from_data(s_flat.as_slice(), &device)
                    .reshape([bs, state_size]);
            let target: Tensor<MyAutodiffBackend, 2> =
                Tensor::<MyAutodiffBackend, 1>::from_data(target_flat.as_slice(), &device)
                    .reshape([bs, action_size]);
            q_state.train(input, target, LEARNING_RATE);
        }

        // --- Action selection ---
        // Q-values for s' (inner backend — no autodiff graph); reused for confidence.
        let s_prime_inner: Tensor<MyBackend, 1> = current_state.as_tensor();
        let q_prime_data = q_state.infer(s_prime_inner).to_data();
        let q_prime = q_prime_data.as_slice::<f32>().unwrap();

        // Decayed epsilon-greedy: explore with the current (annealing) probability.
        let epsilon = EPS_END
            + (EPS_START - EPS_END) * (-(q_state.train_steps() as f64) / EPS_DECAY_STEPS).exp();
        let exploring = rng.random::<f64>() < epsilon;
        let next_action_index = if exploring {
            rng.random_range(0..action_size)
        } else {
            q_prime
                .iter()
                .enumerate()
                // unwrap_or(Equal) keeps a stray NaN from panicking the argmax
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
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
                        let best_price = *best_price;
                        commands.spawn(OrderBundle {
                            order: Order {
                                amount: *amount as f64,
                                max_price_per_unit: best_price,
                                company: Some(entity),
                                resource: resource_id,
                            },
                            time_to_live: TimeToLive(100),
                        });
                        marketplace.statistics.company_orders_placed += 1;
                    }
                }
            }
            CompanyActionEnum::SellResource(resource_idx, amount) => {
                if let Some(&resource_id) = sorted_resource_ids.get(*resource_idx) {
                    // Cap the offer to what the company actually holds right now.
                    let available = stock.resources.get(&resource_id).copied().unwrap_or(0.0);
                    let offer_amount = (*amount as f64).min(available);
                    if offer_amount > 0.0 {
                        if let Some(Some((_, best_price))) =
                            marketplace.order_index.get(&resource_id)
                        {
                            let best_price = *best_price;
                            commands.spawn(OfferBundle {
                                offer: Offer {
                                    amount: offer_amount,
                                    price_per_unit: best_price,
                                    company: Some(entity),
                                    resource: resource_id,
                                },
                                time_to_live: TimeToLive(100),
                            });
                            marketplace.statistics.company_offers_placed += 1;
                        }
                    }
                }
            }
        }

        // Record (s', a') so the next tick can compute the transition (s', a', r', s'')
        *last_tick_money = LastTickMoney(money.0);
        *last_state = current_state;
        *last_action = CompanyAction(next_action_index);

        // Accumulate the learning curve: raw per-tick PnL into the in-progress episode.
        let record = training.companies.entry(entity).or_default();
        record.current_return += raw_reward;
        training.epsilon = epsilon;
    }
}
