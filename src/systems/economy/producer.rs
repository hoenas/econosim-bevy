use bevy::prelude::*;

use crate::components::common::TimeToLive;
use crate::components::economy::offer::Offer;
use crate::components::economy::offer::OfferBundle;
use crate::components::economy::producer::ProducerConfig;

pub fn manage_producers(mut commands: Commands, mut producers: Query<&mut ProducerConfig>) {
    for mut producer in producers.iter_mut() {
        if producer.ticks_since_last_offer % producer.ticks_between_offers == 0 {
            // Create a new offer
            commands.spawn(OfferBundle {
                offer: Offer {
                    company: None,
                    resource: producer.resource,
                    amount: producer.offer_amount,
                    price_per_unit: producer.offer_price,
                },
                time_to_live: TimeToLive(producer.ticks_between_offers),
            });
            producer.ticks_since_last_offer = 0;
        }
        producer.ticks_since_last_offer += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::economy::offer::Offer;
    use crate::components::economy::producer::ProducerConfig;

    fn app() -> App {
        let mut a = App::new();
        a.add_systems(Update, manage_producers);
        a
    }

    fn config(ticks_since: usize, ticks_between: usize) -> ProducerConfig {
        ProducerConfig {
            resource: 0,
            offer_amount: 10.0,
            offer_price: 5.0,
            ticks_between_offers: ticks_between,
            ticks_since_last_offer: ticks_since,
        }
    }

    #[test]
    fn spawns_offer_on_interval_tick() {
        let mut app = app();
        app.world_mut().spawn(config(0, 3));
        app.update();
        let mut q = app.world_mut().query::<&Offer>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn no_offer_between_intervals() {
        let mut app = app();
        app.world_mut().spawn(config(1, 3));
        app.update();
        let mut q = app.world_mut().query::<&Offer>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn offer_has_correct_resource_and_price() {
        let mut app = app();
        app.world_mut().spawn(config(0, 3));
        app.update();
        let mut q = app.world_mut().query::<&Offer>();
        let offer = q.iter(app.world()).next().unwrap();
        assert_eq!(offer.resource, 0);
        assert_eq!(offer.amount, 10.0);
        assert_eq!(offer.price_per_unit, 5.0);
        assert!(offer.company.is_none());
    }
}
