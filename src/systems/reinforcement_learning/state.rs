use burn::prelude::*;

#[derive(PartialEq, Clone)]
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

impl CompanyState {}

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
        let mut values: Vec<f64> = vec![];
        values.append(&mut self.stock.clone());
        values.push(self.money as f64);
        values.append(&mut self.price_index.clone());
        values.append(&mut self.order_index.clone());
        values.append(&mut self.processor_counts.clone());
        Tensor::from_data(values.as_slice(), &Default::default())
    }
}
