#![recursion_limit = "512"]
use bevy::camera::Camera2d;
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use burn::backend::wgpu::WgpuDevice;
use burn::prelude::Module;
use burn::record::{CompactRecorder, Recorder};
use econosim_bevy::components::common::Name;
use econosim_bevy::components::common::RenderColor;
use econosim_bevy::components::economy::company::Company;
use econosim_bevy::components::economy::company::CompanyMarker;
use econosim_bevy::components::economy::consumer::Consumer;
use econosim_bevy::components::economy::consumer::ConsumerConfig;
use econosim_bevy::components::economy::money::LastTickMoney;
use econosim_bevy::components::economy::money::Money;
use econosim_bevy::components::economy::offer::Offer;
use econosim_bevy::components::economy::order::Order;
use econosim_bevy::components::economy::processor::Productive;
use econosim_bevy::components::economy::processor::{Processor, Processors};
use econosim_bevy::components::economy::producer::Producer;
use econosim_bevy::components::economy::producer::ProducerConfig;
use econosim_bevy::components::economy::production_speed::ProductionSpeed;
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::stock::Stock;
use econosim_bevy::components::reinforcement_learning::action::CompanyAction;
use econosim_bevy::components::reinforcement_learning::company_state::CompanyState;
use econosim_bevy::components::reinforcement_learning::nn::NeuralNetwork;
use econosim_bevy::resources::economy::common::Currency;
use econosim_bevy::resources::economy::common::Id;
use econosim_bevy::resources::economy::marketplace::Marketplace;
use econosim_bevy::resources::economy::processor::ProcessorPrice;
use econosim_bevy::resources::economy::recipes::Recipes;
use econosim_bevy::resources::economy::resources::Resources;
use econosim_bevy::resources::reinforcement_learning::action_space::ActionSpace;
use econosim_bevy::resources::reinforcement_learning::q_networks::{CompanyQState, QNetworkStore};
use econosim_bevy::resources::save_state::{SaveLoadState, SaveMetadata, SaveRecipe};
use econosim_bevy::resources::sim_history::SimHistory;
use econosim_bevy::resources::sim_state::SimState;
use econosim_bevy::systems::common::update_time_to_live;
use econosim_bevy::systems::economy::consumer::manage_consumers;
use econosim_bevy::systems::economy::draw_companies::{clean_company_texts, draw_companies};
use econosim_bevy::systems::economy::draw_marketplace::{clean_marketplace_texts, draw_marketplace};
use econosim_bevy::systems::economy::marketplace::execute_orders;
use econosim_bevy::systems::economy::marketplace::{update_order_index, update_price_index};
use econosim_bevy::systems::economy::processor::update_processors;
use econosim_bevy::systems::economy::producer::manage_producers;
use econosim_bevy::systems::reinforcement_learning::company_controller::control_companies;
use econosim_bevy::systems::ui::dashboard::{draw_dashboard, update_sim_history};
use std::collections::HashMap;
use std::path::PathBuf;

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

fn sync_sim_time(sim_state: Res<SimState>, mut time_virtual: ResMut<Time<Virtual>>) {
    if sim_state.paused {
        time_virtual.pause();
    } else {
        time_virtual.unpause();
    }
}

fn do_reset(world: &mut World) {
    *world.resource_mut::<SimHistory>() = SimHistory::default();
    *world.resource_mut::<Marketplace>() = Marketplace::default();

    let to_despawn: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, Or<(With<Order>, With<Offer>)>>();
        q.iter(world).collect()
    };
    for e in to_despawn {
        world.despawn(e);
    }

    let resource_ids: Vec<Id> = world
        .resource::<Resources>()
        .resources
        .keys()
        .copied()
        .collect();
    let recipes_len = world.resource::<Recipes>().recipes.len();

    let companies: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<CompanyMarker>>();
        q.iter(world).collect()
    };
    for entity in companies {
        let stock_resources: HashMap<Id, f64> =
            resource_ids.iter().map(|&id| (id, 10000.0)).collect();
        world.entity_mut(entity).insert((
            Money(1000.0),
            LastTickMoney(1000.0),
            Stock { resources: stock_resources },
            Processors {
                processors: vec![Processor {
                    production_speed: ProductionSpeed(1.0),
                    productive: Productive(true),
                    recipe: Recipe(0),
                }],
            },
            CompanyState::new(resource_ids.len(), recipes_len),
            CompanyAction(0),
        ));
    }

    let producers: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<ProducerConfig>>();
        q.iter(world).collect()
    };
    for entity in producers {
        world.entity_mut(entity).get_mut::<ProducerConfig>().unwrap().ticks_since_last_offer = 0;
    }
    let consumers: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<ConsumerConfig>>();
        q.iter(world).collect()
    };
    for entity in consumers {
        world.entity_mut(entity).get_mut::<ConsumerConfig>().unwrap().ticks_since_last_order = 0;
    }
}

fn reset_simulation(world: &mut World) {
    if !world.resource::<SimState>().reset_requested {
        return;
    }
    world.resource_mut::<SimState>().reset_requested = false;
    do_reset(world);
}

fn step_simulation(world: &mut World) {
    let should_step = {
        let s = world.resource::<SimState>();
        s.paused && s.step_requested
    };
    if should_step {
        world.resource_mut::<SimState>().step_requested = false;
        world.run_schedule(FixedUpdate);
    }
}

fn save_simulation(world: &mut World) {
    let (requested, name) = {
        let s = world.resource::<SaveLoadState>();
        (s.save_requested, s.name.clone())
    };
    if !requested {
        return;
    }
    world.resource_mut::<SaveLoadState>().save_requested = false;

    let save_dir = PathBuf::from("saves").join(&name);
    if let Err(e) = std::fs::create_dir_all(&save_dir) {
        error!("Cannot create save directory: {}", e);
        return;
    }

    // Collect world metadata
    let mut resources: Vec<(usize, String)> = world
        .resource::<Resources>()
        .resources
        .iter()
        .map(|(&id, n)| (id, n.clone()))
        .collect();
    resources.sort_by_key(|(id, _)| *id);

    let mut recipes: Vec<SaveRecipe> = world
        .resource::<Recipes>()
        .recipes
        .iter()
        .map(|(&id, r)| {
            let mut ingredients: Vec<(usize, f64)> =
                r.ingredients.iter().map(|(&i, &a)| (i, a)).collect();
            ingredients.sort_by_key(|(i, _)| *i);
            let mut products: Vec<(usize, f64)> =
                r.products.iter().map(|(&i, &a)| (i, a)).collect();
            products.sort_by_key(|(i, _)| *i);
            SaveRecipe {
                id,
                name: r.name.clone(),
                ingredients,
                products,
                production_speed: r.production_speed,
            }
        })
        .collect();
    recipes.sort_by_key(|r| r.id);

    let mut companies: Vec<(Entity, String)> = {
        let mut q = world.query::<(Entity, &Name, &CompanyMarker)>();
        q.iter(world).map(|(e, n, _)| (e, n.0.clone())).collect()
    };
    companies.sort_by_key(|(_, n)| n.clone());

    let res_count = resources.len();
    let rec_count = recipes.len();
    let state_size = CompanyState::new(res_count, rec_count).get_size(res_count, rec_count);
    let action_size = world.resource::<ActionSpace>().actions.len();

    let metadata = SaveMetadata {
        resources,
        recipes,
        companies: companies.iter().map(|(_, n)| n.clone()).collect(),
        state_size,
        action_size,
    };
    match serde_json::to_string_pretty(&metadata) {
        Ok(json) => {
            if let Err(e) = std::fs::write(save_dir.join("metadata.json"), json) {
                error!("Failed to write metadata: {}", e);
                return;
            }
        }
        Err(e) => {
            error!("Metadata serialization failed: {}", e);
            return;
        }
    }

    // Save each company's network
    let q_store = world.non_send_resource::<QNetworkStore>();
    let recorder = CompactRecorder::new();
    for (i, (entity, company_name)) in companies.iter().enumerate() {
        if let Some(q_state) = q_store.0.get(entity) {
            if let Some(network) = &q_state.network {
                let path = save_dir.join(format!("company_{}", i));
                if let Err(e) = recorder.record(network.clone().into_record(), path) {
                    error!("Failed to save network for '{}': {}", company_name, e);
                }
            }
        }
    }
    info!("Saved simulation to '{}'", name);
}

fn load_simulation(world: &mut World) {
    let (requested, name) = {
        let s = world.resource::<SaveLoadState>();
        (s.load_requested, s.name.clone())
    };
    if !requested {
        return;
    }
    world.resource_mut::<SaveLoadState>().load_requested = false;

    let save_dir = PathBuf::from("saves").join(&name);
    let meta: SaveMetadata = match std::fs::read(save_dir.join("metadata.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(m) => m,
        None => {
            error!("Failed to read save '{}'", name);
            return;
        }
    };

    // Validate that network dimensions match the current world
    let res_count = world.resource::<Resources>().resources.len();
    let rec_count = world.resource::<Recipes>().recipes.len();
    let cur_state = CompanyState::new(res_count, rec_count).get_size(res_count, rec_count);
    let cur_action = world.resource::<ActionSpace>().actions.len();
    if meta.state_size != cur_state || meta.action_size != cur_action {
        error!(
            "Save '{}' incompatible: state/action ({}/{}) != current ({}/{})",
            name, meta.state_size, meta.action_size, cur_state, cur_action
        );
        return;
    }
    let state_size = meta.state_size;
    let action_size = meta.action_size;

    // Reset the simulation before loading so companies are in a clean state
    do_reset(world);

    // Build name → entity map
    let name_to_entity: HashMap<String, Entity> = {
        let mut q = world.query::<(Entity, &Name, &CompanyMarker)>();
        q.iter(world).map(|(e, n, _)| (n.0.clone(), e)).collect()
    };

    // Load each company's network into QNetworkStore
    let device = WgpuDevice::default();
    let recorder = CompactRecorder::new();
    for (i, company_name) in meta.companies.iter().enumerate() {
        let path = save_dir.join(format!("company_{}", i));
        match recorder.load(path, &device) {
            Ok(record) => {
                let network =
                    NeuralNetwork::new(&device, state_size, action_size).load_record(record);
                if let Some(&entity) = name_to_entity.get(company_name) {
                    let mut q_store = world.non_send_resource_mut::<QNetworkStore>();
                    let q_state = q_store
                        .0
                        .entry(entity)
                        .or_insert_with(|| CompanyQState::new(&device, state_size, action_size));
                    q_state.network = Some(network);
                } else {
                    error!("Company '{}' not found in current world", company_name);
                }
            }
            Err(e) => error!("Failed to load network {}: {}", i, e),
        }
    }
    info!("Loaded simulation from '{}'", name);
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
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .insert_resource(SimHistory::default())
        .insert_resource(SimState::default())
        .insert_resource(SaveLoadState::default())
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
        .add_systems(PreUpdate, sync_sim_time)
        .add_systems(
            PostUpdate,
            (reset_simulation, step_simulation, save_simulation, load_simulation).chain(),
        )
        .add_systems(FixedUpdate, control_companies)
        .add_systems(FixedUpdate, update_sim_history.after(control_companies))
        .add_systems(EguiPrimaryContextPass, draw_dashboard)
        .add_systems(FixedUpdate, update_processors)
        .add_systems(Update, (clean_company_texts, draw_companies))
        .add_systems(FixedUpdate, (manage_consumers, manage_producers))
        .add_systems(FixedUpdate, update_time_to_live)
        .add_systems(
            FixedUpdate,
            (update_price_index, update_order_index, execute_orders),
        )
        .add_systems(Update, (clean_marketplace_texts, draw_marketplace))
        .run();
}
