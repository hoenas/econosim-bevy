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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::TimeToLive;

    fn app() -> App {
        let mut a = App::new();
        a.add_systems(Update, update_time_to_live);
        a
    }

    #[test]
    fn ttl_decrements_each_tick() {
        let mut app = app();
        let e = app.world_mut().spawn(TimeToLive(5)).id();
        app.update();
        assert_eq!(app.world().get::<TimeToLive>(e).unwrap().0, 4);
    }

    #[test]
    fn entity_despawned_when_ttl_reaches_zero() {
        let mut app = app();
        let e = app.world_mut().spawn(TimeToLive(1)).id();
        app.update();
        assert!(!app.world().entities().contains(e));
    }

    #[test]
    fn entity_survives_while_ttl_above_zero() {
        let mut app = app();
        let e = app.world_mut().spawn(TimeToLive(2)).id();
        app.update();
        assert!(app.world().entities().contains(e));
    }
}
