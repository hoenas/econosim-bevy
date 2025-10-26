use bevy::color::Color;
use bevy::prelude::Component;
use rand::prelude::*;

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct TimeToLive(pub usize);

#[derive(Component)]
pub struct RenderColor(pub Color);

impl Default for RenderColor {
    fn default() -> Self {
        let mut rng = rand::rng();
        RenderColor(Color::srgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
        ))
    }
}
