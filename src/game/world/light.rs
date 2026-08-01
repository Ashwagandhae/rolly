use macroquad::prelude::*;

use crate::game::assets::Assets;

use super::svg::{read_svg, SvgShape};

#[derive(Debug, Clone)]
pub struct Light {
    pub pos: Vec2,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct LightGroup {
    pub lights: Vec<(Light, f32)>,
}

pub fn shape_to_light(shape: &SvgShape) -> Light {
    match shape {
        SvgShape::Circle(circle) => {
            // we don't care about the rotation of the circle
            Light {
                pos: circle.pos.into(),
                radius: circle.r,
            }
        }
        _ => panic!("Only circles are supported for lights"),
    }
}

pub fn load_light(assets: &Assets, light: &str) -> LightGroup {
    let svg = &assets.lights[light];
    let (size, items) = read_svg(&svg);
    let lights = items
        .into_iter()
        .map(|item| shape_to_light(&item.shape))
        .map(|light| Light {
            pos: light.pos - size / 2.0,
            ..light
        })
        .map(|light| (light, 1.0))
        .collect::<Vec<_>>();
    LightGroup { lights }
}

#[derive(Debug, Clone)]
pub struct Power {
    pub strength: f32,
    pub radius: f32,
}
