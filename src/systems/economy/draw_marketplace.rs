use crate::resources::economy::marketplace::Marketplace;
use bevy::{prelude::*, sprite::Text2dShadow};
use itertools::Itertools;

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

pub fn draw_marketplace(mut commands: Commands, marketplace: Res<Marketplace>) {
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
