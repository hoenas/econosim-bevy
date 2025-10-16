use crate::components::economy::stock::Stock;
use bevy::{prelude::*, sprite::Text2dShadow};

pub fn draw_companies(mut commands: Commands, query: Query<&Stock>) {
    let text_font = TextFont {
        font_size: 50.0,
        ..default()
    };
    let text_justification = Justify::Center;
    // commands.spawn(Camera2d);
    if query.is_empty() {
        println!("No companies to draw.");
        return;
    }
    for stock in query.iter() {
        println!("Doing something");
        let stock_text = format!(
            "{}",
            stock
                .resources
                .iter()
                .map(|(id, amount)| format!("{}: {:.1}", id, amount))
                .collect::<Vec<String>>()
                .join(", ")
        );
        commands.spawn((
            Text2d::new(stock_text),
            text_font.clone(),
            TextLayout::new_with_justify(text_justification),
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
            Text2dShadow::default(),
        ));
    }
}
