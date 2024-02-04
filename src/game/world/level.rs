use macroquad::prelude::*;
use rapier2d::dynamics::RigidBodyHandle;

use crate::consts::*;
use crate::game::assets::Assets;
use crate::game::world::floor::Material;
use crate::game::world::svg::{read_svg, SvgShape};
use crate::game::world::thing::ThingInfo;

use super::draw::{get_camera_rect, meter_to_pixel, pos_in_camera};
use super::floor::spawn_floor;

use super::thing::spawn_thing;
use super::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelId(pub usize);

impl LevelId {
    fn valid(&self, assets: &Assets) -> bool {
        assets.levels.contains_key(&self.0)
    }
    fn next(&self, assets: &Assets) -> Option<LevelId> {
        let next = LevelId(self.0 + 1);
        if next.valid(assets) {
            Some(next)
        } else {
            None
        }
    }
    fn prev(&self, assets: &Assets) -> Option<LevelId> {
        if self.0 == 0 {
            return None;
        }
        let prev = LevelId(self.0 - 1);
        if prev.valid(assets) {
            Some(prev)
        } else {
            None
        }
    }
    pub fn first() -> LevelId {
        LevelId(0)
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
}

impl std::default::Default for Markers {
    fn default() -> Self {
        Self {
            start: vec2(0.0, 0.0),
            end: vec2(0.0, 0.0),
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
pub fn load_level(assets: &Assets, world: &mut World, level: LevelId) {
    load_level_at_pos(assets, world, level, vec2(LEVEL_X, LEVEL_Y))
}
pub fn load_level_at_pos(assets: &Assets, world: &mut World, level: LevelId, pos: Vec2) {
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

pub fn unload_level(world: &mut World, level: LevelId) {
    world.levels.remove(&level).unwrap();
    let remove_entities = world
        .entities
        .query_mut::<&LevelId>()
        .into_iter()
        .filter(|(_, entity_level)| **entity_level == level)
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in remove_entities {
        if let Some(body) = world
            .entities
            .entity(entity)
            .ok()
            .and_then(|e| e.get::<&RigidBodyHandle>())
        {
            world.physics_world.remove_body(*body);
        }
        world.entities.despawn(entity).unwrap();
    }
}

pub fn update_loaded_levels_alive(assets: &Assets, world: &mut World) {
    // only load/unload levels if the player is alive
    if !world.player.alive() {
        return;
    }
    update_loaded_levels(assets, world);
}

pub fn update_loaded_levels(assets: &Assets, world: &mut World) {
    let levels_to_load = world
        .levels
        .iter()
        .flat_map(|(level, pos)| find_adjacent_levels_to_load(assets, world, *level, *pos))
        .filter_map(|x| x)
        .collect::<Vec<_>>();
    let levels_to_unload = find_levels_to_unload(assets, world);
    for &(level, pos) in levels_to_load.iter() {
        if !world.levels.contains_key(&level) {
            load_level_at_pos(assets, world, level, pos);
        }
    }
    for level in levels_to_unload {
        if !levels_to_load.iter().any(|(l, _)| *l == level) {
            unload_level(world, level);
        }
    }
}

fn find_adjacent_levels_to_load(
    assets: &Assets,
    world: &World,
    level: LevelId,
    pos: Vec2,
) -> [Option<(LevelId, Vec2)>; 2] {
    let (info, _) = &assets.levels[&level.0];

    let load_queries: [(Option<LevelId>, Vec2, fn(&LevelInfo, &LevelInfo) -> Vec2); 2] = [
        (level.next(assets), info.markers.end, |info, next_info| {
            info.markers.end - next_info.markers.start
        }),
        (level.prev(assets), info.markers.start, |info, prev_info| {
            info.markers.start - prev_info.markers.end
        }),
    ];
    load_queries.map(|(edge_level, end_pos, new_pos)| {
        if !pos_in_camera(world, pos + end_pos) {
            return None;
        }
        let edge_level = edge_level?;
        let (new_info, _) = &assets.levels.get(&edge_level.0)?;
        let new_pos = pos + new_pos(info, new_info);
        Some((edge_level, new_pos))
    })
}

fn find_levels_to_unload(assets: &Assets, world: &World) -> Vec<LevelId> {
    world
        .levels
        .iter()
        .filter_map(|(level, pos)| {
            let info = &assets.levels[&level.0].0;
            let rect = Rect::new(pos.x, pos.y, info.dims.x, info.dims.y);
            if !get_camera_rect(world).overlaps(&rect) {
                Some(*level)
            } else {
                None
            }
        })
        .collect()
}
