use crate::components::economy::stock::Stock;
use crate::components::{common::Name, economy::money::Money};
use crate::resources::economy::common::Currency;
use crate::resources::economy::resources::Resources;
use bevy::prelude::*;

pub fn control_companies(
    query: Query<(&Name, &Stock, &Money)>,
    resources: Res<Resources>,
    currency: Res<Currency>,
) {
    for (name, stock, money) in query.iter() {
        // TODO: Do stuff
    }
}
