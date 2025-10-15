use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use econosim_bevy::components::common::{Id, Name};
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::currency::Currency;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::resource::Resource;
use econosim_bevy::components::economy::stock::Stock;

fn create_resources(mut commands: Commands) {
    commands.spawn((Resource {}, Name("Water".to_string()), Id(0)));
    commands.spawn((Resource {}, Name("Dirt".to_string()), Id(1)));
    commands.spawn((Resource {}, Name("Wood".to_string()), Id(2)));
    commands.spawn((Resource {}, Name("Coal".to_string()), Id(3)));
}

fn create_recipes(mut commands: Commands) {
    let mut ingredients = HashMap::new();
    ingredients.insert(0, 10.0); // Water
    ingredients.insert(1, 10.0); // Dirt
    ingredients.insert(2, 10.0); // Wood

    let mut products = HashMap::new();
    products.insert(3, 1.0); // Coal

    commands.spawn((
        Recipe {
            ingredients,
            products,
            production_speed: 1.0,
        },
        Name("Coal".to_string()),
        Id(0),
    ));
}

fn create_processors(mut commands: Commands) {
    commands.spawn((
        Processor {
            production_speed: 1.0,
            productive: true,
            recipe: 0,
        },
        Name("Processor Wood".to_string()),
        Id(0),
    ));
}

fn create_companies(mut commands: Commands) {
    commands.spawn((
        Company {
            stock: Stock::new(),
            currency: Currency(1000.0),
            processors: Processors {
                processors: vec![Processor {
                    production_speed: 1.0,
                    productive: true,
                    recipe: 0,
                }],
            },
        },
        Name("Company A".to_string()),
        Id(0),
    ));
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
