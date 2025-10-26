use crate::components::economy::offer::Offer;
use crate::components::economy::order::Order;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::Resources;
use bevy::{prelude::*, sprite::Text2dShadow};

use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct MarkerMarketplaceText();

pub fn clean_marketplace_texts(
    mut commands: Commands,
    query: Query<Entity, With<MarkerMarketplaceText>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn draw_marketplace(
    mut commands: Commands,
    marketplace: Res<Marketplace>,
    resources: Res<Resources>,
    orders: Query<&Order>,
    offers: Query<&Offer>,
) {
    let text_font = TextFont {
        font_size: 14.0,
        ..default()
    };
    let mut marketplace_text = String::from("Marketplace Statistics:\n");
    marketplace_text.push_str(&format!(
        "Orders placed:           {}\n",
        marketplace.statistics.company_orders_placed
    ));
    marketplace_text.push_str(&format!(
        "Offers placed:           {}\n",
        marketplace.statistics.company_offers_placed
    ));
    marketplace_text.push_str(&format!(
        "Partly fulfilled orders: {}\n",
        marketplace.statistics.company_orders_partly_fulfilled
    ));
    marketplace_text.push_str(&format!(
        "Partly fulfilled offers: {}\n",
        marketplace.statistics.company_offers_partly_fulfilled
    ));
    marketplace_text.push_str(&format!(
        "Orders fulfilled:        {}\n",
        marketplace.statistics.company_orders_fulfilled
    ));
    marketplace_text.push_str(&format!(
        "Offers fulfilled:        {}\n",
        marketplace.statistics.company_offers_fulfilled
    ));
    marketplace_text.push_str(&format!(
        "Open orders:             {}\n",
        orders.iter().len()
    ));
    marketplace_text.push_str(&format!(
        "Open offers:             {}\n",
        offers.iter().len()
    ));
    marketplace_text.push_str("Price index:\n");
    for (resource_id, order_price_tuple) in marketplace.price_index.iter() {
        let mut resource_price = 0.0;
        if order_price_tuple.is_some() {
            resource_price = order_price_tuple.unwrap().1;
        }
        let resource_name = resources.resources.get(resource_id).unwrap();
        marketplace_text.push_str(&format!(" - {}: {}\n", resource_name, resource_price));
    }
    marketplace_text.push_str("Order index:\n");
    for (resource_id, order_price_tuple) in marketplace.order_index.iter() {
        let mut resource_price = 0.0;
        if order_price_tuple.is_some() {
            resource_price = order_price_tuple.unwrap().1;
        }
        let resource_name = resources.resources.get(resource_id).unwrap();
        marketplace_text.push_str(&format!(" - {}: {}\n", resource_name, resource_price));
    }
    commands.spawn((
        Text2d::new(marketplace_text),
        text_font.clone(),
        TextLayout::new_with_justify(Justify::Left),
        Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
        TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
        Text2dShadow::default(),
        MarkerMarketplaceText::default(),
    ));
}
