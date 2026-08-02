use crate::resources::economy::common::Id;
use crate::resources::economy::resources::*;
use bevy::prelude::Resource;
use std::collections::HashMap;

pub struct Recipe {
    pub name: String,
    pub ingredients: HashMap<Id, f64>,
    pub products: HashMap<Id, f64>,
    pub production_speed: f64,
}

/// Builds a recipe from ingredient/product `(resource id, amount)` slices.
fn recipe(name: &str, ingredients: &[(Id, f64)], products: &[(Id, f64)]) -> Recipe {
    Recipe {
        name: name.to_string(),
        ingredients: ingredients.iter().copied().collect(),
        products: products.iter().copied().collect(),
        production_speed: 1.0,
    }
}

#[derive(Resource)]
pub struct Recipes {
    pub recipes: HashMap<Id, Recipe>,
}

impl Default for Recipes {
    fn default() -> Self {
        // (recipe id, recipe). Ids must stay contiguous from 0: the action space enumerates
        // BuyProcessor(0..recipe_count). Coal is a mined raw (see producers), so there is no
        // recipe for it. Rough intended margins are noted per chain; final balance depends on
        // the producer offer prices and consumer bids set up separately.
        let defs: Vec<(Id, Recipe)> = vec![
            // ── Wood chain ──
            (0, recipe("Planks", &[(WOOD, 2.0)], &[(PLANKS, 3.0)])),
            (1, recipe("Charcoal", &[(WOOD, 3.0)], &[(CHARCOAL, 2.0)])),

            // ── Metal chain (fuelled by Coal) ──
            (2, recipe("Iron", &[(IRON_ORE, 3.0), (COAL, 1.0)], &[(IRON, 2.0)])),
            (3, recipe("Steel", &[(IRON, 2.0), (COAL, 1.0)], &[(STEEL, 1.0)])),
            (4, recipe("Tools", &[(STEEL, 1.0), (PLANKS, 2.0)], &[(TOOLS, 3.0)])),
            (5, recipe("Machinery", &[(STEEL, 2.0), (TOOLS, 2.0)], &[(MACHINERY, 1.0)])),

            // ── Construction / glass ──
            (6, recipe("Glass", &[(SAND, 2.0), (COAL, 1.0)], &[(GLASS, 2.0)])),
            (7, recipe("Bricks", &[(DIRT, 3.0), (COAL, 1.0)], &[(BRICKS, 4.0)])),
            (8, recipe("Concrete", &[(STONE, 2.0), (WATER, 1.0), (SAND, 1.0)], &[(CONCRETE, 3.0)])),
            (9, recipe("Bottles", &[(GLASS, 1.0)], &[(BOTTLES, 2.0)])),
            (10, recipe(
                "Building Materials",
                &[(BRICKS, 2.0), (GLASS, 1.0), (PLANKS, 2.0)],
                &[(BUILDING_MATERIALS, 1.0)],
            )),

            // ── Wood finished good ──
            (11, recipe("Furniture", &[(PLANKS, 3.0), (TOOLS, 1.0)], &[(FURNITURE, 1.0)])),

            // ── Food chain ──
            (12, recipe("Wheat", &[(WATER, 2.0), (DIRT, 2.0)], &[(WHEAT, 3.0)])),
            (13, recipe("Flour", &[(WHEAT, 2.0)], &[(FLOUR, 1.0)])),
            (14, recipe("Bread", &[(FLOUR, 1.0), (WATER, 1.0)], &[(BREAD, 2.0)])),

            // ── Petrochemical ──
            (15, recipe("Plastic", &[(CRUDE_OIL, 2.0)], &[(PLASTIC, 3.0)])),
        ];

        Recipes {
            recipes: defs.into_iter().collect(),
        }
    }
}
