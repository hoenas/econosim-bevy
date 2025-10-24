use crate::components::common::TimeToLive;
use bevy::prelude::Query;
use bevy::prelude::*;

pub fn update_time_to_live(mut commands: Commands, mut orders: Query<(Entity, &mut TimeToLive)>) {
    for (order, mut entity) in orders.iter_mut() {
        entity.0 -= 1;
        if entity.0 == 0 {
            commands.entity(order).despawn();
        }
    }
}
