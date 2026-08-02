use crate::resources::economy::common::Id;
use bevy::prelude::Resource;
use std::collections::HashMap;

pub type ResourceHandle = usize;

// Stable resource ids. Recipes reference these by name so the two files can't drift.
// ── Raw materials (mined/harvested — supplied by producers) ──
pub const WATER: Id = 0;
pub const DIRT: Id = 1;
pub const WOOD: Id = 2;
pub const COAL: Id = 3;
pub const IRON_ORE: Id = 4;
pub const SAND: Id = 5;
pub const STONE: Id = 6;
pub const CRUDE_OIL: Id = 7;
// ── Intermediates (produced by recipes from raws) ──
pub const PLANKS: Id = 8;
pub const CHARCOAL: Id = 9;
pub const IRON: Id = 10;
pub const STEEL: Id = 11;
pub const GLASS: Id = 12;
pub const BRICKS: Id = 13;
pub const CONCRETE: Id = 14;
pub const WHEAT: Id = 15;
pub const FLOUR: Id = 16;
pub const PLASTIC: Id = 17;
pub const TOOLS: Id = 18;
// ── Finished goods (bought by consumers) ──
pub const FURNITURE: Id = 19;
pub const MACHINERY: Id = 20;
pub const BREAD: Id = 21;
pub const BOTTLES: Id = 22;
pub const BUILDING_MATERIALS: Id = 23;

#[derive(Resource)]
pub struct Resources {
    pub resources: HashMap<Id, String>,
}

impl Default for Resources {
    fn default() -> Self {
        let resources = [
            (WATER, "Water"),
            (DIRT, "Dirt"),
            (WOOD, "Wood"),
            (COAL, "Coal"),
            (IRON_ORE, "Iron Ore"),
            (SAND, "Sand"),
            (STONE, "Stone"),
            (CRUDE_OIL, "Crude Oil"),
            (PLANKS, "Planks"),
            (CHARCOAL, "Charcoal"),
            (IRON, "Iron"),
            (STEEL, "Steel"),
            (GLASS, "Glass"),
            (BRICKS, "Bricks"),
            (CONCRETE, "Concrete"),
            (WHEAT, "Wheat"),
            (FLOUR, "Flour"),
            (PLASTIC, "Plastic"),
            (TOOLS, "Tools"),
            (FURNITURE, "Furniture"),
            (MACHINERY, "Machinery"),
            (BREAD, "Bread"),
            (BOTTLES, "Bottles"),
            (BUILDING_MATERIALS, "Building Materials"),
        ]
        .into_iter()
        .map(|(id, name)| (id, name.to_string()))
        .collect();
        Resources { resources }
    }
}
