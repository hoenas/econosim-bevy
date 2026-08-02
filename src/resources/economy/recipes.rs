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
        // (recipe id, recipe). Rough intended margins are noted per chain; final balance
        // depends on the producer offer prices and consumer bids set up separately.
        let defs: Vec<(Id, Recipe)> = vec![
            // Legacy placeholder — kept for compatibility; superseded once Coal is mined as a
            // raw. Currently loss-making (inputs cost far more than Coal sells for).
            (0, recipe("Coal (synthetic)", &[(WATER, 10.0), (DIRT, 10.0), (WOOD, 10.0)], &[(COAL, 1.0)])),

            // ── Wood chain ──
            (1, recipe("Planks", &[(WOOD, 2.0)], &[(PLANKS, 3.0)])),
            (2, recipe("Charcoal", &[(WOOD, 3.0)], &[(CHARCOAL, 2.0)])),

            // ── Metal chain (fuelled by Coal) ──
            (3, recipe("Iron", &[(IRON_ORE, 3.0), (COAL, 1.0)], &[(IRON, 2.0)])),
            (4, recipe("Steel", &[(IRON, 2.0), (COAL, 1.0)], &[(STEEL, 1.0)])),
            (5, recipe("Tools", &[(STEEL, 1.0), (PLANKS, 2.0)], &[(TOOLS, 3.0)])),
            (6, recipe("Machinery", &[(STEEL, 2.0), (TOOLS, 2.0)], &[(MACHINERY, 1.0)])),

            // ── Construction / glass ──
            (7, recipe("Glass", &[(SAND, 2.0), (COAL, 1.0)], &[(GLASS, 2.0)])),
            (8, recipe("Bricks", &[(DIRT, 3.0), (COAL, 1.0)], &[(BRICKS, 4.0)])),
            (9, recipe("Concrete", &[(STONE, 2.0), (WATER, 1.0), (SAND, 1.0)], &[(CONCRETE, 3.0)])),
            (10, recipe("Bottles", &[(GLASS, 1.0)], &[(BOTTLES, 2.0)])),
            (11, recipe(
                "Building Materials",
                &[(BRICKS, 2.0), (GLASS, 1.0), (PLANKS, 2.0)],
                &[(BUILDING_MATERIALS, 1.0)],
            )),

            // ── Wood finished good ──
            (12, recipe("Furniture", &[(PLANKS, 3.0), (TOOLS, 1.0)], &[(FURNITURE, 1.0)])),

            // ── Food chain ──
            (13, recipe("Wheat", &[(WATER, 2.0), (DIRT, 2.0)], &[(WHEAT, 3.0)])),
            (14, recipe("Flour", &[(WHEAT, 2.0)], &[(FLOUR, 1.0)])),
            (15, recipe("Bread", &[(FLOUR, 1.0), (WATER, 1.0)], &[(BREAD, 2.0)])),

            // ── Petrochemical ──
            (16, recipe("Plastic", &[(CRUDE_OIL, 2.0)], &[(PLASTIC, 3.0)])),
        ];

        Recipes {
            recipes: defs.into_iter().collect(),
        }
    }
}
