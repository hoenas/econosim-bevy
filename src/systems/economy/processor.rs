use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use crate::resources::economy::recipes::Recipes;
use bevy::prelude::Query;
use bevy::prelude::Res;

pub fn update_processors(
    // Let all processors produce their products based on their production speed and the available ingredients in the stock
    mut producers: Query<(&Processors, &mut Stock)>,
    recipes: Res<Recipes>,
) {
    for (processors, mut stock) in producers.iter_mut() {
        for processor in &processors.processors {
            if !processor.productive.0 {
                continue;
            }

            if let Some(recipe) = recipes.recipes.get(&processor.recipe.0) {
                // Check if we have enough ingredients in the stock
                let mut can_produce = true;
                for (ingredient_id, &amount) in &recipe.ingredients {
                    let available_amount =
                        stock.resources.get(ingredient_id).cloned().unwrap_or(0.0);
                    if available_amount < amount * processor.production_speed.0 {
                        can_produce = false;
                        break;
                    }
                }

                if can_produce {
                    // Deduct ingredients from the stock
                    for (ingredient_id, &amount) in &recipe.ingredients {
                        let entry = stock.resources.entry(*ingredient_id).or_insert(0.0);
                        *entry -= amount * processor.production_speed.0;
                    }

                    // Add products to the stock
                    for (product_id, &amount) in &recipe.products {
                        let entry = stock.resources.entry(*product_id).or_insert(0.0);
                        *entry += amount * processor.production_speed.0;
                    }
                }
            }
        }
    }
}
