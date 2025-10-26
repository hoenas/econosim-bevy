use bevy::camera::Camera2d;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::*;
use econosim_bevy::components::common::Name;
use econosim_bevy::components::common::RenderColor;
use econosim_bevy::components::common::TimeToLive;
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::company::CompanyMarker;
use econosim_bevy::components::economy::money::Money;
use econosim_bevy::components::economy::offer::{Offer, OfferBundle};
use econosim_bevy::components::economy::order::{Order, OrderBundle};
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::stock::Stock;
use econosim_bevy::resources::economy::common::Id;
use econosim_bevy::resources::economy::marketplace::Marketplace;
use econosim_bevy::resources::economy::recipes::Recipes;
use econosim_bevy::resources::economy::resources::Resources;
use econosim_bevy::systems::common::update_time_to_live;
use econosim_bevy::systems::economy::draw_companies::{
    clean_company_texts, draw_companies, draw_plot,
};
use econosim_bevy::systems::economy::draw_marketplace::{
    clean_marketplace_texts, draw_marketplace,
};
use econosim_bevy::systems::economy::marketplace::execute_orders;
use econosim_bevy::systems::economy::marketplace::{update_order_index, update_price_index};
use econosim_bevy::systems::economy::processor::update_processors;
use rand::prelude::*;
use std::collections::HashMap;

fn create_resources(mut commands: Commands) {
    commands.insert_resource(Resources::default());
}

fn create_recipes(mut commands: Commands) {
    commands.insert_resource(Recipes::default());
}

fn create_marketplace(mut commands: Commands) {
    commands.insert_resource(Marketplace::default());
}

fn create_companies(mut commands: Commands) {
    let mut resources: HashMap<Id, f64> = HashMap::new();
    resources.insert(0, 10000.0);
    resources.insert(1, 10000.0);
    resources.insert(2, 10000.0);
    for company in 0..3 {
        commands.spawn((Company {
            stock: Stock {
                resources: resources.clone(),
            },
            money: Money(1000.0),
            processors: Processors {
                processors: vec![Processor {
                    production_speed: ProductionSpeed(1.0),
                    productive: Productive(true),
                    recipe: Recipe(0),
                }],
            },
            name: Name(format!("Company{}", company)),
            marker: CompanyMarker::default(),
            color: RenderColor::default(),
        },));
    }
}

fn create_offers_and_orders(
    mut commands: Commands,
    companies: Query<(Entity, &Money)>,
    resources: Res<Resources>,
) {
    let mut rng = rand::rng();
    for (company, _) in companies {
        for resource in resources.resources.keys() {
            commands.spawn(OfferBundle {
                offer: Offer {
                    amount: rng.random_range(0.0..1.0) * 100.0,
                    price_per_unit: rng.random_range(0.0..1.0) * 10.0,
                    company: Some(company),
                    resource: *resource,
                },
                time_to_live: TimeToLive(10000),
            });
            commands.spawn(OrderBundle {
                order: Order {
                    amount: rng.random_range(0.0..1.0) * 10000.0,
                    max_price_per_unit: rng.random_range(0.0..1.0) * 10.0,
                    company: Some(company),
                    resource: *resource,
                },
                time_to_live: TimeToLive(10000),
            });
        }
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
            (
                create_resources,
                create_recipes.after(create_resources),
                create_marketplace,
                create_companies.after(create_recipes),
                create_offers_and_orders.after(create_companies),
            ),
        )
        // Update companies
        .add_systems(Update, update_processors)
        .add_systems(Update, (clean_company_texts, draw_companies, draw_plot))
        // Update market
        .add_systems(Update, update_time_to_live)
        .add_systems(
            Update,
            (update_price_index, update_order_index, execute_orders),
        )
        .add_systems(Update, (clean_marketplace_texts, draw_marketplace))
        .run();
}
