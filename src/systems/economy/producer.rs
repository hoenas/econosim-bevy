use bevy::prelude::*;

use crate::components::economy::offer::Offer;
use crate::components::economy::producer::ProducerConfig;

pub fn manage_producers(mut commands: Commands, mut producers: Query<&mut ProducerConfig>) {
    for mut producer in producers.iter_mut() {
        producer.ticks_since_last_offer += 1;
        if producer.ticks_between_offers == producer.ticks_between_offers {
            // Create a new order
            commands.spawn(Offer {
                company: None,
                resource: producer.resource,
                amount: producer.offer_amount,
                price_per_unit: producer.offer_price,
            });
        }
    }
}
