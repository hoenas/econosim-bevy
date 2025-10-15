use crate::components::common::Id;
use crate::components::common::Name;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Resource {
    pub id: Id,
    pub name: Name,
}

impl Default for Resource {
    fn default() -> Self {
        Resource {
            id: Id(0),
            name: Name("Unnamed Resource".to_string()),
        }
    }
}
