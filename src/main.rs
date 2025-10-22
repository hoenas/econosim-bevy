use bevy::camera::Camera2d;
use bevy::prelude::*;
use econosim_bevy::components::common::Name;
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::currency::Currency;
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::stock::Stock;
use econosim_bevy::resources::economy::common::Id;
use econosim_bevy::resources::economy::recipes::Recipes;
use econosim_bevy::resources::economy::resources::Resources;
use econosim_bevy::systems::economy::draw_companies::{clean_company_texts, draw_companies};
use econosim_bevy::systems::economy::processor::update_processors;
use std::collections::HashMap;

fn create_resources(mut commands: Commands) {
    commands.insert_resource(Recipes::default());
}

fn create_recipes(mut commands: Commands) {
    commands.insert_resource(Resources::default());
}

fn create_companies(mut commands: Commands) {
    let mut resources: HashMap<Id, f64> = HashMap::new();
    resources.insert(0, 1000.0);
    resources.insert(1, 1000.0);
    resources.insert(2, 1000.0);
    for company in 0..3 {
        commands.spawn((Company {
            stock: Stock {
                resources: resources.clone(),
            },
            currency: Currency(1000.0),
            processors: Processors {
                processors: vec![Processor {
                    production_speed: ProductionSpeed(1.0),
                    productive: Productive(true),
                    recipe: Recipe(0),
                }],
            },
            name: Name(format!("Company{}", company)),
        },));
    }
}

#[derive(Component)]
struct MyCameraMarker;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        MyCameraMarker,
        Transform::from_xyz(0.0, 0.0, 1000.0),
        GlobalTransform::default(),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .add_systems(
            Startup,
            (create_resources, create_recipes, create_companies),
        )
        .add_systems(Update, update_processors)
        .add_systems(Update, (clean_company_texts, draw_companies))
        .run();
}
