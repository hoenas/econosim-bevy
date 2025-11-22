use crate::components::economy::money::LastTickMoney;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use crate::components::{common::Name, economy::money::Money};
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::recipes::Recipes;
use crate::resources::economy::resources::Resources;
use crate::resources::reinforcement_learning::action_space::{ActionSpace, CompanyAction};
use crate::systems::reinforcement_learning::state::CompanyState;
use bevy::prelude::*;
use itertools::Itertools;

pub fn control_companies(
    query: Query<(&Name, &Stock, &Processors, &Money, &LastTickMoney)>,
    resources: Res<Resources>,
    recipes: Res<Recipes>,
    action_space: Res<ActionSpace>,
    marketplace: Res<Marketplace>,
) {
    for (name, stock, processors, money, last_tick_money) in query.iter() {
        let reward = money.0 - last_tick_money.0;
        let mut processor_counts = vec![];
        // Get processor counts
        for recipe_id in recipes.recipes.iter().map(|x| x.0).sorted() {
            let mut processor_count = 0;
            for processor in processors
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

        // TODO: Ask agent for next action

        // // Act according to agent decision
        // match action_space.actions[action] {
        //     CompanyAction::Nothing => {
        //         // do nothing
        //         return;
        //     }
        //     CompanyAction::BuyProcessor(recipe) => {
        //         if recipe_data.recipes.len() <= recipe {
        //             return;
        //         }
        //         self.buy_processor(recipe, processor_price, &recipe_data);
        //     }
        //     CompanyAction::SellProcessor(recipe) => {
        //         // Search for processor with given recipe
        //         for (processor_handle, processor) in self.processors.iter().enumerate() {
        //             if processor.recipe == recipe {
        //                 self.sell_processor(processor_handle, processor_price);
        //                 return;
        //             }
        //         }
        //     }
        //     CompanyAction::BuyResource(resource, amount) => {
        //         // Buy resource to current best price
        //         if market_data.price_index[&resource].is_none() {
        //             return;
        //         }
        //         self.place_order(
        //             resource,
        //             amount as f64,
        //             market_data.price_index[&resource].unwrap().1,
        //         );
        //     }
        //     CompanyAction::SellResource(resource, amount) => {
        //         // Sell resource to current best price
        //         if market_data.order_index[&resource].is_none() {
        //             return;
        //         }
        //         self.place_offer(
        //             resource,
        //             amount as f64,
        //             market_data.order_index[&resource].unwrap().1,
        //         );
        //     }
        // }
    }
}
