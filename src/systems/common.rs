use crate::components::common::TimeToLive;
use bevy::prelude::Query;
use bevy::prelude::*;

pub fn update_time_to_live(mut commands: Commands, mut orders: Query<(Entity, &mut TimeToLive)>) {
    for (order_entity, mut time_to_live) in orders.iter_mut() {
        time_to_live.0 -= 1;
        if time_to_live.0 == 0 {
            commands.entity(order_entity).despawn();
        }
    }
}
