use bevy::ecs::component::Component;
use burn::prelude::*;

pub const MONEY_SCALE: f64 = 1_000.0;
const PRICE_SCALE: f64 = 10.0;
const STOCK_SCALE: f64 = 1_000.0;
const PROCESSOR_SCALE: f64 = 10.0;

#[derive(PartialEq, Clone, Component)]
pub struct CompanyState {
    // Stockpile
    pub stock: Vec<f64>,
    // Currency
    pub money: f64,
    // Price and order index
    pub price_index: Vec<f64>,
    pub order_index: Vec<f64>,
    // Processor counts
    pub processor_counts: Vec<f64>,
}

impl CompanyState {
    pub fn new(resource_count: usize, recipe_count: usize) -> CompanyState {
        CompanyState {
            stock: vec![0.0; resource_count],
            money: 0.0,
            price_index: vec![0.0; resource_count],
            order_index: vec![0.0; resource_count],
            processor_counts: vec![0.0; recipe_count],
        }
    }

    /// Initial observation for a freshly created company holding `money` cash and nothing
    /// else. Seeding money here (rather than leaving it at 0) means the first transition's
    /// net-worth reward is 0 instead of a spurious spike equal to the starting cash.
    pub fn initial(resource_count: usize, recipe_count: usize, money: f64) -> CompanyState {
        let mut state = CompanyState::new(resource_count, recipe_count);
        state.money = money;
        state
    }

    /// Mark-to-market net worth: cash + stock valued at current buy prices + processors at
    /// their purchase cost. Used as the RL reward basis so that converting cash into assets
    /// (buying resources or processors) nets to ~0 reward rather than looking like a loss.
    pub fn net_worth(&self, processor_price: f64) -> f64 {
        let stock_value: f64 = self
            .stock
            .iter()
            .zip(&self.price_index)
            .map(|(units, price)| units * price)
            .sum();
        let processor_value = self.processor_counts.iter().sum::<f64>() * processor_price;
        self.money + stock_value + processor_value
    }

    /// Flattens the state into the scaled feature vector the network consumes.
    /// Shared by `as_tensor` and the replay buffer so both use identical scaling/layout.
    pub fn as_vec(&self) -> Vec<f32> {
        let mut values: Vec<f32> = Vec::new();
        for &s in &self.stock {
            values.push((s / STOCK_SCALE) as f32);
        }
        values.push((self.money / MONEY_SCALE) as f32);
        for &p in &self.price_index {
            values.push((p / PRICE_SCALE) as f32);
        }
        for &o in &self.order_index {
            values.push((o / PRICE_SCALE) as f32);
        }
        for &c in &self.processor_counts {
            values.push((c / PROCESSOR_SCALE) as f32);
        }
        values
    }

    pub fn as_tensor<B: Backend>(&self) -> Tensor<B, 1> {
        Tensor::from_data(self.as_vec().as_slice(), &Default::default())
    }

    /// Length of the flat feature vector for the given world dimensions.
    pub fn size(resource_count: usize, recipe_count: usize) -> usize {
        1 + 3 * resource_count + recipe_count
    }
}
