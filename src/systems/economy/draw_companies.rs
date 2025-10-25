use crate::components::economy::stock::Stock;
use crate::components::{common::Name, economy::money::Money};
use crate::resources::economy::resources::Resources;
use bevy::{prelude::*, sprite::Text2dShadow};
use itertools::Itertools;

use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct MarkerStockText();

pub fn clean_company_texts(mut commands: Commands, query: Query<Entity, With<MarkerStockText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn draw_companies(
    mut commands: Commands,
    query: Query<(&Name, &Stock, &Money)>,
    resources: Res<Resources>,
) {
    let text_font = TextFont {
        font_size: 14.0,
        ..default()
    };
    if query.is_empty() {
        println!("No companies to draw.");
        return;
    }
    for (index, (name, stock, money)) in query.iter().enumerate() {
        let mut stock_text = format!("{}: Money: {} ", name.0, money.0);
        for (resource_id, resource_name) in resources.resources.iter().sorted() {
            let amount = stock.resources.get(&resource_id).cloned().unwrap_or(0.0);
            stock_text.push_str(&format!(" {}: {:.2}", resource_name, amount));
        }

        commands.spawn((
            Text2d::new(stock_text),
            text_font.clone(),
            TextLayout::new_with_justify(Justify::Left),
            Transform::from_translation(Vec3::new(0.0, 20.0 * (index as f32), 0.0)),
            TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
            Text2dShadow::default(),
            MarkerStockText::default(),
        ));
    }
}
