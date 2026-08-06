use macroquad::prelude::*;

use crate::{
    consts::{LIGHT_FLICKER_GROW_SPEED, LIGHT_FLICKER_INTERVAL},
    game::assets::Assets,
};

use super::svg::{SvgShape, read_svg};

#[derive(Debug, Clone)]
pub struct Light {
    pub pos: Vec2,
    pub radius: f32,
}
pub struct LightRipple {}

#[derive(Debug, Clone)]
pub struct LightGroup {
    pub lights: Vec<(Light, LightState)>,
}
#[derive(Debug, Clone)]
pub enum LightState {
    Flicker(FlickerState),
    Ripple(RippleState),
}
#[derive(Debug, Clone)]
pub enum FlickerState {
    Growing(f32),
    Shrinking(f32),
    Off,
}
#[derive(Debug, Clone)]
pub struct RippleState {
    pub strength: f32,
}
impl RippleState {
    pub fn update(&mut self, dt: f32) {
        self.strength += LIGHT_FLICKER_GROW_SPEED * dt;
        self.strength = self.strength.clamp(0.0, 1.0);
    }
}
impl FlickerState {
    pub fn update(&mut self, dt: f32) {
        *self = match self {
            Self::Growing(strength) => {
                let new_strength = *strength + LIGHT_FLICKER_GROW_SPEED * dt;
                if new_strength > 1.0 {
                    Self::Shrinking(1.0 - (new_strength - 1.0))
                } else {
                    Self::Growing(new_strength)
                }
            }
            Self::Shrinking(strength) => {
                let new_strength = *strength - LIGHT_FLICKER_GROW_SPEED * dt;
                if new_strength < 0.0 {
                    Self::Off
                } else {
                    Self::Shrinking(new_strength)
                }
            }
            Self::Off => {
                let prob = get_frame_time() / LIGHT_FLICKER_INTERVAL;
                let start_flicker = rand::gen_range(0.0, 1.0) < prob;
                if start_flicker {
                    Self::Growing(0.0)
                } else {
                    Self::Off
                }
            }
        }
    }
}
impl LightState {
    pub fn strength(&self) -> f32 {
        match self {
            LightState::Flicker(flicker) => match flicker {
                FlickerState::Growing(s) | FlickerState::Shrinking(s) => *s,
                FlickerState::Off => 0.0,
            },
            LightState::Ripple(RippleState { strength }) => *strength,
        }
    }
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

pub fn load_light(assets: &Assets, light: &str, init_state: LightState) -> LightGroup {
    let svg = &assets.lights[light];
    let (size, items) = read_svg(&svg);
    let lights = items
        .into_iter()
        .map(|item| shape_to_light(&item.shape))
        .map(|light| Light {
            pos: light.pos - size / 2.0,
            ..light
        })
        .map(|light| (light, init_state.clone()))
        .collect::<Vec<_>>();
    LightGroup { lights }
}

#[derive(Debug, Clone)]
pub struct Power {
    pub strength: f32,
    pub radius: f32,
}
