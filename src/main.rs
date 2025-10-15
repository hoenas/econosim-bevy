use bevy::prelude::*;
use econosim_bevy::components::common::{ComponentId, Id, Name};
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::currency::Currency;
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::{Ingredients, Products, Recipe};
use econosim_bevy::components::economy::resource::Resource;
use econosim_bevy::components::economy::stock::Stock;
use std::collections::HashMap;

fn create_resources(mut commands: Commands) {
    commands.spawn(Resource {
        name: Name("Water".to_string()),
        id: Id(0),
    });
    commands.spawn(Resource {
        name: Name("Dirt".to_string()),
        id: Id(1),
    });
    commands.spawn(Resource {
        name: Name("Wood".to_string()),
        id: Id(2),
    });
    commands.spawn(Resource {
        name: Name("Coal".to_string()),
        id: Id(3),
    });
}

fn create_recipes(mut commands: Commands) {
    let mut ingredients: HashMap<ComponentId, f64> = HashMap::new();
    ingredients.insert(0, 10.0); // Water
    ingredients.insert(1, 10.0); // Dirt
    ingredients.insert(2, 10.0); // Wood

    let mut products: HashMap<ComponentId, f64> = HashMap::new();
    products.insert(3, 1.0); // Coal

    commands.spawn(Recipe {
        ingredients: Ingredients(ingredients),
        products: Products(products),
        production_speed: ProductionSpeed(1.0),
        name: Name("Coal".to_string()),
        id: Id(0),
    });
}

fn create_processors(mut commands: Commands) {
    commands.spawn((
        Processor {
            production_speed: ProductionSpeed(1.0),
            productive: Productive(true),
            recipe: Id(0),
        },
        Name("Processor Wood".to_string()),
        Id(0),
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
                recipe: Id(0),
            }],
        },
        name: Name("Company A".to_string()),
        id: Id(0),
    },));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (create_resources, create_recipes, create_companies),
        )
        .run();
}
