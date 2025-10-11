use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use econosim_bevy::components::common::{Id, Name};
use econosim_bevy::components::economy::recipe::Recipe;
use econosim_bevy::components::economy::resource::Resource;

fn create_resources(mut commands: Commands) {
    commands.spawn((Resource {}, Name("Water".to_string()), Id(0)));
    commands.spawn((Resource {}, Name("Dirt".to_string()), Id(1)));
    commands.spawn((Resource {}, Name("Wood".to_string()), Id(2)));
    commands.spawn((Resource {}, Name("Coal".to_string()), Id(3)));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, create_resources)
        .run();
}
