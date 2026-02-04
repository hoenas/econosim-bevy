use crate::components::economy::money::LastTickMoney;
use crate::components::economy::processor::{Processor, Processors, Productive};
use crate::components::economy::production_speed::ProductionSpeed;
use crate::components::economy::recipe::Recipe;
use crate::components::economy::stock::Stock;
use crate::components::reinforcement_learning::action::CompanyAction;
use crate::components::reinforcement_learning::company_state::CompanyState;
use crate::components::{common::Name, economy::money::Money};
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::processor::ProcessorPrice;
use crate::resources::economy::recipes::Recipes;
use crate::resources::economy::resources::Resources;
use crate::resources::reinforcement_learning::action_space::{ActionSpace, CompanyActionEnum};
use crate::resources::reinforcement_learning::backend::MyAutodiffBackend;
use bevy::prelude::*;
use burn::Tensor;
use itertools::Itertools;

pub fn control_companies(
    mut commands: Commands,
    mut companies: Query<(
        &Name,
        &Stock,
        &mut Processors,
        &mut Money,
        &LastTickMoney,
        &CompanyState,
        &CompanyAction,
    )>,
    resources: Res<Resources>,
    recipes: Res<Recipes>,
    processor_price: Res<ProcessorPrice>,
    action_space: Res<ActionSpace>,
    marketplace: Res<Marketplace>,
) {
    for (name, stock, mut processors, mut money, last_tick_money, last_state, last_action) in
        companies.iter_mut()
    {
        let mut processor_counts = vec![];
        // Get processor counts
        for recipe_id in recipes.recipes.iter().map(|x| x.0).sorted() {
            let mut processor_count = 0;
            for _ in processors
                .processors
                .iter()
                .filter(|x| x.recipe.0 == *recipe_id)
            {
                processor_count += 1;
            }
            processor_counts.push(processor_count as f64);
        }
        // Get order index
        let mut order_index = vec![];
        for resource in marketplace.order_index.iter().map(|x| *x.0).sorted() {
            let order = marketplace.order_index.get(&resource).unwrap();
            match order {
                Some(order) => order_index.push(order.1),
                None => order_index.push(0.0),
            }
        }
        // Get order index
        let mut price_index = vec![];
        for resource in marketplace.price_index.iter().map(|x| *x.0).sorted() {
            let price = marketplace.price_index.get(&resource).unwrap();
            match price {
                Some(price) => price_index.push(price.1),
                None => price_index.push(0.0),
            }
        }
        // Get stock
        let mut stock_vec = vec![];
        for resource in stock.resources.iter().map(|x| *x.0).sorted() {
            let amount = stock.resources.get(&resource).unwrap();
            stock_vec.push(*amount);
        }
        let company_state = CompanyState {
            money: money.0,
            order_index: order_index,
            price_index: price_index,
            processor_counts: processor_counts,
            stock: stock_vec,
        };

        // Deep Q learning algorith
        let device = burn::backend::wgpu::WgpuDevice::default();
        let mut q_network = crate::components::reinforcement_learning::nn::NeuralNetwork::new(
            &device,
            company_state.get_size(resources.resources.len(), recipes.recipes.len()),
            action_space.actions.len(),
        );
        let alpha = 0.01;
        let gamma = 0.01;
        let s_plus_1 = company_state.as_tensor();
        let s = last_state.as_tensor();
        let a = last_action.0;
        let r_s_a = money.0 - last_tick_money.0;
        let output = q_network.forward(s.clone()).to_data();
        let current_q_values = output.as_slice().unwrap();

        let q_s_a: f64 = current_q_values[a];
        let max_q_s_a_plus_1: f64 = q_network
            .forward(s_plus_1.clone())
            .max()
            .to_data()
            .as_slice()
            .unwrap()[0];

        let new_q_value = q_s_a + alpha * (r_s_a + gamma * max_q_s_a_plus_1 - q_s_a);
        let mut target_q_values = current_q_values.to_vec();
        target_q_values[a] = new_q_value;
        q_network.train(
            s,
            Tensor::<MyAutodiffBackend, 1>::from_data(&target_q_values[..], &device),
        );

        // Select new action
        let next_action_index: i32 = q_network
            .forward(s_plus_1.clone())
            .argmax(1)
            .to_data()
            .as_slice()
            .unwrap()[0];
        let next_action = action_space
            .actions
            .get(next_action_index as usize)
            .unwrap();

        // Act according to agent decision
        match next_action {
            CompanyActionEnum::Nothing => {
                // do nothing
                return;
            }
            CompanyActionEnum::BuyProcessor(recipe) => {
                if recipes.recipes.len() <= *recipe {
                    return;
                }
                // Check if the company has enough funding
                if money.0 < processor_price.0 {
                    return;
                }
                money.0 -= processor_price.0;
                // Create processor
                let processor = Processor {
                    production_speed: ProductionSpeed(1.0),
                    productive: Productive(true),
                    recipe: Recipe(*recipe),
                };
                processors.processors.push(processor);
            }
            CompanyActionEnum::SellProcessor(recipe) => {
                // Search for processor with given recipe
                if recipes.recipes.len() <= *recipe {
                    return;
                }
                for (i, processor) in processors.processors.iter_mut().enumerate() {
                    if processor.recipe.0 == *recipe {
                        processors.processors.remove(i);
                        return;
                    }
                }
            }
            CompanyActionEnum::BuyResource(resource, amount) => {
                // // Buy resource to current best price
                // if market_data.price_index[&resource].is_none() {
                //     return;
                // }
                // self.place_order(
                //     resource,
                //     amount as f64,
                //     market_data.price_index[&resource].unwrap().1,
                // );
            }
            CompanyActionEnum::SellResource(resource, amount) => {
                // // Sell resource to current best price
                // if market_data.order_index[&resource].is_none() {
                //     return;
                // }
                // self.place_offer(
                //     resource,
                //     amount as f64,
                //     market_data.order_index[&resource].unwrap().1,
                // );
            }
        }
    }
}
