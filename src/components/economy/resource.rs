use crate::components::common::Name;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Resource {
    pub name: Name,
}

impl Default for Resource {
    fn default() -> Self {
        Resource {
            name: Name("Unnamed Resource".to_string()),
        }
    }
}
