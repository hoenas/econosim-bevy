use crate::components::common::RenderColor;
use crate::components::economy::stock::Stock;
use crate::components::{common::Name, economy::money::Money};
use crate::resources::economy::resources::Resources;
use bevy::{prelude::*, sprite::Text2dShadow};
use itertools::{Itertools, Position};

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
    query: Query<(&Name, &Stock, &Money, &RenderColor)>,
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
    for (index, (name, stock, money, color)) in query.iter().enumerate() {
        let mut stock_text = format!("{}: Money: {:.2} ", name.0, money.0);
        for (resource_id, resource_name) in resources.resources.iter().sorted() {
            let amount = stock.resources.get(&resource_id).cloned().unwrap_or(0.0);
            stock_text.push_str(&format!(" {}: {:.2}", resource_name, amount));
        }

        commands.spawn((
            Text2d::new(stock_text),
            TextColor(color.0),
            text_font.clone(),
            TextLayout::new_with_justify(Justify::Left),
            Transform::from_translation(Vec3::new(0.0, 20.0 * (index as f32), 0.0)),
            TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
            Text2dShadow::default(),
            MarkerStockText::default(),
        ));
    }
}

pub fn draw_plot(
    mut commands: Commands,
    query: Query<(&Name, &Stock, &RenderColor)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    resources: Res<Resources>,
) {
    let bounds = Rectangle::new(800.0, 400.0);
    let origin_x = bounds.size().x / -2.0;
    let origin_y = bounds.size().y / -2.0;

    let time_since_start = time.elapsed().as_secs_f32();

    for (_, stock, color) in query.iter() {
        for (resource, _) in resources.resources.iter() {
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(1.0))),
                MeshMaterial2d(materials.add(color.0)),
                Transform::from_xyz(
                    origin_x + bounds.size().x * time_since_start / 50.0,
                    origin_y + *stock.resources.get(&resource).unwrap_or(&0.0) as f32 / 100.0,
                    0.0,
                ),
            ));
        }
    }
}
