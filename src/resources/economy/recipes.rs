use crate::resources::economy::common::Id;
use bevy::prelude::Resource;
use std::collections::HashMap;

pub struct Recipe {
    pub name: String,
    pub ingredients: HashMap<Id, f64>,
    pub products: HashMap<Id, f64>,
    pub production_speed: f64,
}

#[derive(Resource)]
pub struct Recipes {
    pub recipes: HashMap<Id, Recipe>,
}

impl Default for Recipes {
    fn default() -> Self {
        let mut recipes: HashMap<Id, Recipe> = HashMap::new();
        let mut ingredients: HashMap<Id, f64> = HashMap::new();

        ingredients.insert(0, 10.0); // Water
        ingredients.insert(1, 10.0); // Dirt
        ingredients.insert(2, 10.0); // Wood

        let mut products: HashMap<Id, f64> = HashMap::new();
        products.insert(3, 1.0); // Coal

        recipes.insert(
            0,
            Recipe {
                name: "Coal".to_string(),
                ingredients,
                products,
                production_speed: 1.0,
            },
        );

        Recipes { recipes }
    }
}
