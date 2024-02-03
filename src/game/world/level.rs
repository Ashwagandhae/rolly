use macroquad::prelude::*;

use crate::game::assets::Assets;
use crate::game::ui::settings::Settings;
use crate::game::world::floor::Material;
use crate::game::world::svg::{read_svg, SvgShape};
use crate::game::world::thing::ThingInfo;

use super::draw::{meter_to_pixel, pos_in_camera};
use super::floor::spawn_floor;
use super::thing::spawn_thing;
use super::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelId(pub usize);

impl LevelId {
    fn next(&self) -> Self {
        LevelId(self.0 + 1)
    }
    fn prev(&self) -> Option<Self> {
        if self.0 == 0 {
            None
        } else {
            Some(LevelId(self.0 - 1))
        }
    }
}

#[derive(Debug, Clone)]
pub struct LevelInfo {
    pub dims: Vec2,
    pub markers: Markers,
}

#[derive(Debug, Clone)]
pub struct Markers {
    pub start: Vec2,
    pub end: Vec2,
    pub original_spawn: Option<Vec2>,
}

impl std::default::Default for Markers {
    fn default() -> Self {
        Self {
            start: vec2(0.0, 0.0),
            end: vec2(0.0, 0.0),
            original_spawn: None,
        }
    }
}

impl LevelInfo {
    pub fn parse(svg: &str) -> Self {
        let mut markers = Markers::default();
        let (size, items) = read_svg(svg);
        for item in items {
            match item.shape {
                SvgShape::Circle(circle) => {
                    let radius: usize = meter_to_pixel(circle.r).round() as usize;
                    if radius != 50 {
                        continue;
                    }
                    match item.color {
                        0x0000FF => markers.original_spawn = Some(circle.pos),
                        0x00FF00 => markers.start = circle.pos,
                        0xFF0000 => markers.end = circle.pos,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Self {
            dims: size,
            markers,
        }
    }
}

pub fn load_level(assets: &Assets, world: &mut World, level: LevelId, pos: Vec2) {
    let (_, svg) = &assets.levels[&level.0];
    let (_, items) = read_svg(svg);
    for item in items {
        match item.shape {
            SvgShape::Rect(rect) => {
                let thing_info = ThingInfo::new_rect(rect.pos, rect.rotate, rect.dims, item.color);
                spawn_thing(assets, world, thing_info, level, pos);
            }
            SvgShape::Circle(circle) => {
                let thing_info =
                    ThingInfo::new_circle(circle.pos, circle.rotate, circle.r, item.color);
                spawn_thing(assets, world, thing_info, level, pos);
            }
            SvgShape::Path(path) => {
                let material = Material::from_hex_color(item.color);
                spawn_floor(assets, world, path.vertices, material, level, pos);
            }
        }
    }
    world.levels.insert(level, pos);
}

pub fn update_loaded_levels(assets: &Assets, settings: &Settings, world: &mut World) {
    for (level, pos) in world.levels.clone() {
        load_adjacent_levels(assets, settings, world, level, pos);
    }
}

fn load_adjacent_levels(
    assets: &Assets,
    settings: &Settings,
    world: &mut World,
    level: LevelId,
    pos: Vec2,
) {
    let (info, _) = &assets.levels[&level.0];
    let (next_level, end_pos) = (level.next(), info.markers.end);
    if !world.levels.contains_key(&next_level) && pos_in_camera(settings, world, pos + end_pos) {
        let Some((new_info, _)) = &assets.levels.get(&next_level.0) else {
            return;
        };
        let new_pos = dbg!(pos) + dbg!(info.markers.end) - dbg!(new_info.markers.start);
        load_level(assets, world, next_level, new_pos);
    }
}
