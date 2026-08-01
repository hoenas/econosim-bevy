#![recursion_limit = "512"]
use bevy::camera::Camera2d;
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use econosim_bevy::components::common::Name;
use econosim_bevy::components::common::RenderColor;
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::company::CompanyMarker;
use econosim_bevy::components::economy::consumer::Consumer;
use econosim_bevy::components::economy::consumer::ConsumerConfig;
use econosim_bevy::components::economy::money::LastTickMoney;
use econosim_bevy::components::economy::money::Money;
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::producer::Producer;
use econosim_bevy::components::economy::producer::ProducerConfig;
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::stock::Stock;
use econosim_bevy::components::reinforcement_learning::action::CompanyAction;
use econosim_bevy::components::reinforcement_learning::company_state::CompanyState;
use econosim_bevy::resources::economy::common::Currency;
use econosim_bevy::resources::economy::common::Id;
use econosim_bevy::resources::economy::marketplace::Marketplace;
use econosim_bevy::resources::economy::processor::ProcessorPrice;
use econosim_bevy::resources::economy::recipes::Recipes;
use econosim_bevy::resources::economy::resources::Resources;
use econosim_bevy::resources::reinforcement_learning::action_space::ActionSpace;
use econosim_bevy::resources::reinforcement_learning::q_networks::QNetworkStore;
use econosim_bevy::systems::common::update_time_to_live;
use econosim_bevy::systems::economy::consumer::manage_consumers;
use econosim_bevy::systems::economy::draw_companies::{clean_company_texts, draw_companies};
use econosim_bevy::systems::economy::draw_marketplace::{clean_marketplace_texts, draw_marketplace};
use econosim_bevy::systems::economy::marketplace::execute_orders;
use econosim_bevy::systems::economy::marketplace::{update_order_index, update_price_index};
use econosim_bevy::systems::economy::processor::update_processors;
use econosim_bevy::systems::economy::producer::manage_producers;
use econosim_bevy::resources::sim_history::SimHistory;
use econosim_bevy::systems::reinforcement_learning::company_controller::control_companies;
use econosim_bevy::systems::ui::dashboard::{draw_dashboard, update_sim_history};
use std::collections::HashMap;

fn create_resources(mut commands: Commands) {
    commands.insert_resource(Resources::default());
}

fn create_recipes(mut commands: Commands) {
    commands.insert_resource(Recipes::default());
    commands.insert_resource(ProcessorPrice(100.0));
}

fn create_marketplace(mut commands: Commands) {
    commands.insert_resource(Marketplace::default());
}

fn create_currency(mut commands: Commands) {
    commands.insert_resource(Currency {
        name: String::from("Euro"),
        unit: '€',
    });
}

fn create_companies(mut commands: Commands, resources: Res<Resources>, recipes: Res<Recipes>) {
    let mut stock_resources: HashMap<Id, f64> = HashMap::new();
    for (resource, _) in resources.resources.iter() {
        stock_resources.insert(*resource, 10000.0);
    }

    for company in 0..3 {
        commands.spawn(Company {
            stock: Stock {
                resources: stock_resources.clone(),
            },
            money: Money(1000.0),
            last_tick_money: LastTickMoney(1000.0),
            last_state: CompanyState::new(resources.resources.len(), recipes.recipes.len()),
            last_action: CompanyAction(0),
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
        });
    }
}

fn create_consumers_and_producers(mut commands: Commands) {
    commands.spawn(Producer {
        name: Name(String::from("Water Procucer")),
        config: ProducerConfig {
            resource: 0,
            offer_amount: 10000.0,
            offer_price: 1.0,
            ticks_between_offers: 100,
            ticks_since_last_offer: 0,
        },
    });
    commands.spawn(Producer {
        name: Name(String::from("Dirt Procucer")),
        config: ProducerConfig {
            resource: 1,
            offer_amount: 10000.0,
            offer_price: 0.5,
            ticks_between_offers: 100,
            ticks_since_last_offer: 0,
        },
    });
    commands.spawn(Producer {
        name: Name(String::from("Wood Procucer")),
        config: ProducerConfig {
            resource: 2,
            offer_amount: 10000.0,
            offer_price: 2.0,
            ticks_between_offers: 100,
            ticks_since_last_offer: 0,
        },
    });
    commands.spawn(Consumer {
        name: Name(String::from("Coal Consumer")),
        config: ConsumerConfig {
            resource: 3,
            order_amount: 10000.0,
            order_max_price: 10.0,
            ticks_between_orders: 100,
            ticks_since_last_order: 0,
        },
    });
}

#[derive(Component)]
struct MyCameraMarker;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MyCameraMarker,
        Transform::from_xyz(0.0, 0.0, 1000.0),
        GlobalTransform::default(),
    ));
}

fn create_action_space(mut commands: Commands, resources: Res<Resources>, recipes: Res<Recipes>) {
    commands.insert_resource(ActionSpace::new(
        resources.resources.len(),
        recipes.recipes.len(),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(SimHistory::default())
        .insert_non_send_resource(QNetworkStore::default())
        .add_systems(Startup, setup_camera)
        .add_systems(
            Startup,
            (
                create_resources,
                create_recipes.after(create_resources),
                create_marketplace,
                create_currency,
                create_companies.after(create_recipes),
                create_action_space.after(create_recipes),
                create_consumers_and_producers.after(create_recipes),
            ),
        )
        // Update companies
        .add_systems(Update, control_companies)
        .add_systems(Update, update_sim_history.after(control_companies))
        .add_systems(EguiPrimaryContextPass, draw_dashboard)
        .add_systems(Update, update_processors)
        .add_systems(Update, (clean_company_texts, draw_companies))
        // .add_systems(Update, draw_plot)
        // Manage consumers & producers
        .add_systems(Update, (manage_consumers, manage_producers))
        // Update market
        .add_systems(Update, update_time_to_live)
        .add_systems(
            Update,
            (update_price_index, update_order_index, execute_orders),
        )
        .add_systems(Update, (clean_marketplace_texts, draw_marketplace))
        .run();
}
