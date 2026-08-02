use bevy::ecs::component::Component;
use burn::prelude::*;

const MONEY_SCALE: f64 = 1_000.0;
const PRICE_SCALE: f64 = 10.0;
const STOCK_SCALE: f64 = 1_000.0;
const PROCESSOR_SCALE: f64 = 10.0;

#[derive(PartialEq, Clone, Component)]
pub struct CompanyState {
    // Stockpile
    pub stock: Vec<f64>,
    // Currentcy
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

    pub fn as_tensor<B: Backend>(&self) -> Tensor<B, 1> {
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
        Tensor::from_data(values.as_slice(), &Default::default())
    }

    pub fn get_size(&self, resource_count: usize, recipe_count: usize) -> usize {
        1 + 3 * resource_count + recipe_count
    }
}
