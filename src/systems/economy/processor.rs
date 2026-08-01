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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::economy::processor::{Processor, Productive};
    use crate::components::economy::production_speed::ProductionSpeed;
    use crate::components::economy::recipe::Recipe;
    use bevy::prelude::*;

    // Default Recipes: recipe 0 consumes {0:10, 1:10, 2:10} and produces {3:1}.
    fn app() -> App {
        let mut a = App::new();
        a.init_resource::<Recipes>();
        a.add_systems(Update, update_processors);
        a
    }

    fn make_processor(speed: f64, active: bool) -> Processors {
        Processors {
            processors: vec![Processor {
                production_speed: ProductionSpeed(speed),
                productive: Productive(active),
                recipe: Recipe(0),
            }],
        }
    }

    fn stock_with_ingredients(amount: f64) -> Stock {
        let mut s = Stock::default();
        s.resources.insert(0, amount);
        s.resources.insert(1, amount);
        s.resources.insert(2, amount);
        s
    }

    #[test]
    fn produces_when_ingredients_available() {
        let mut app = app();
        let e = app
            .world_mut()
            .spawn((make_processor(1.0, true), stock_with_ingredients(10.0)))
            .id();
        app.update();
        let stock = app.world().get::<Stock>(e).unwrap();
        assert_eq!(*stock.resources.get(&3).unwrap_or(&0.0), 1.0);
        assert_eq!(*stock.resources.get(&0).unwrap_or(&0.0), 0.0);
    }

    #[test]
    fn does_not_produce_when_ingredients_missing() {
        let mut app = app();
        let e = app
            .world_mut()
            .spawn((make_processor(1.0, true), Stock::default()))
            .id();
        app.update();
        let stock = app.world().get::<Stock>(e).unwrap();
        assert_eq!(*stock.resources.get(&3).unwrap_or(&0.0), 0.0);
    }

    #[test]
    fn inactive_processor_produces_nothing() {
        let mut app = app();
        let e = app
            .world_mut()
            .spawn((make_processor(1.0, false), stock_with_ingredients(10.0)))
            .id();
        app.update();
        let stock = app.world().get::<Stock>(e).unwrap();
        assert_eq!(*stock.resources.get(&3).unwrap_or(&0.0), 0.0);
        assert_eq!(*stock.resources.get(&0).unwrap(), 10.0);
    }

    #[test]
    fn production_scales_with_speed() {
        let mut app = app();
        let e = app
            .world_mut()
            .spawn((make_processor(2.0, true), stock_with_ingredients(20.0)))
            .id();
        app.update();
        let stock = app.world().get::<Stock>(e).unwrap();
        assert_eq!(*stock.resources.get(&3).unwrap_or(&0.0), 2.0);
        assert_eq!(*stock.resources.get(&0).unwrap_or(&0.0), 0.0);
    }
}
