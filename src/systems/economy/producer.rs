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
