use bevy::prelude::*;
use econosim_bevy::components::common::Name;
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::currency::Currency;
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::resource::Resource;
use econosim_bevy::components::economy::stock::Stock;
use econosim_bevy::resources::economy::recipes::Recipes;
use econosim_bevy::resources::economy::resources::Resources;
use econosim_bevy::systems::economy::processor::update_processors;
use std::collections::HashMap;

fn create_resources(mut commands: Commands) {
    commands.insert_resource(Recipes::default());
}

fn create_recipes(mut commands: Commands) {
    commands.insert_resource(Resources::default());
}

fn create_processors(mut commands: Commands) {
    commands.spawn((
        Processor {
            production_speed: ProductionSpeed(1.0),
            productive: Productive(true),
            recipe: Recipe(0),
        },
        Name("Processor Wood".to_string()),
    ));
}

fn create_companies(mut commands: Commands) {
    commands.spawn((Company {
        stock: Stock::default(),
        currency: Currency(1000.0),
        processors: Processors {
            processors: vec![Processor {
                production_speed: ProductionSpeed(1.0),
                productive: Productive(true),
                recipe: Recipe(0),
            }],
        },
        name: Name("Company A".to_string()),
    },));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (create_resources, create_recipes, create_companies),
        )
        .add_systems(Update, update_processors)
        .run();
}
