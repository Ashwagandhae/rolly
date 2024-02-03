use std::f32::consts::PI;

use super::super::floor::VertexDraw;
use super::super::polygon::{shrink_polygon, trimesh_indices_from_polygon};
use super::{draw_texture_centered, draw_trimesh, get_camera_rect, pixel_to_meter};
use crate::consts::*;
use crate::game::assets::{Assets, TileConstraints};
use crate::game::world::thing::ThingDraw;
use crate::game::world::World;
use hecs::Or;
use itertools::Itertools;
use macroquad::prelude::*;
use rapier2d::dynamics::RigidBodyHandle;

pub fn draw(assets: &Assets, world: &World) {
    for (_, (draw, body)) in world
        .entities
        .query::<(Or<&VertexDraw, &ThingDraw>, &RigidBodyHandle)>()
        .iter()
    {
        if let Or::Left(vertex_draw) | Or::Both(vertex_draw, _) = draw {
            match vertex_draw {
                VertexDraw::Tiled(tiled_draw) => {
                    draw_tiled(assets, world, tiled_draw);
                }
                VertexDraw::Liquid(liquid_draw) => {
                    draw_liquid(liquid_draw);
                }
            }
        }
        if let Or::Right(thing_draw) | Or::Both(_, thing_draw) = draw {
            draw_thing(assets, world, thing_draw, body);
        }
    }
}

fn two_points_rect(v1: Vec2, v2: Vec2) -> Rect {
    let start = vec2(v1.x.min(v2.x), v1.y.min(v2.y));
    let end = vec2(v1.x.max(v2.x), v1.y.max(v2.y));
    Rect::new(start.x, start.y, end.x - start.x, end.y - start.y)
}

fn add_rect_padding(rect: Rect, padding: f32) -> Rect {
    Rect::new(
        rect.x - padding,
        rect.y - padding,
        rect.w + padding * 2.0,
        rect.h + padding * 2.0,
    )
}

pub fn draw_tiled(assets: &Assets, world: &World, tiled_draw: &TiledDraw) {
    for ((vertices, indices), color) in tiled_draw.trimeshes.iter().zip(tiled_draw.colors.iter()) {
        draw_trimesh(vertices, indices, *color)
    }
    let vertices = &tiled_draw.trimeshes[0].0;
    for ((_, &v1, &v2, _), (rect, left_offset, textures)) in vertices
        .iter()
        .circular_tuple_windows()
        .zip(tiled_draw.tile_textures.iter())
    {
        if !get_camera_rect(world).overlaps(rect) {
            continue;
        }
        let rotation = Vec2::new(1.0, 0.0).angle_between(v1 - v2);
        let rotation_down = rotation + PI / 2.0;
        let draw_on_line = |dist: f32, texture_file: &str| {
            let down = Vec2::from_angle(rotation_down) * pixel_to_meter(TILE_DOWN);
            let pos = v1 + (v2 - v1).normalize() * dist + down;

            draw_texture_centered(assets, texture_file, pos, rotation, None);
        };
        for (j, texture_file) in textures.iter().rev().enumerate() {
            let dist = j as f32 * pixel_to_meter(TILE_WIDTH)
                + pixel_to_meter(TILE_WIDTH / 2.0)
                + left_offset;
            draw_on_line(dist, texture_file);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiquidDraw {
    pub color: Color,
    pub trimesh: (Vec<Vec2>, Vec<[u32; 3]>),
}

impl LiquidDraw {
    pub fn new(vertices: &[Vec2], color: Color) -> Self {
        let trimesh = (vertices.to_vec(), trimesh_indices_from_polygon(vertices));
        Self { color, trimesh }
    }
}
#[derive(Debug, Clone)]
pub struct TiledDraw {
    pub tile: &'static str,
    pub colors: [Color; 3],
    pub trimeshes: [(Vec<Vec2>, Vec<[u32; 3]>); 3],
    pub tile_textures: Vec<(Rect, f32, Vec<String>)>,
}

impl TiledDraw {
    pub fn new(assets: &Assets, tile: &'static str, colors: [Color; 3], vertices: &[Vec2]) -> Self {
        let shrink_1 = shrink_polygon(&vertices, pixel_to_meter(40.0));
        let shrink_2 = shrink_polygon(&shrink_1, pixel_to_meter(40.0));
        let map = |vertices: Vec<Vec2>| {
            let trimesh_indices = trimesh_indices_from_polygon(&vertices);
            (vertices, trimesh_indices)
        };
        let trimeshes = [map(vertices.to_vec()), map(shrink_1), map(shrink_2)];

        let tile_textures = vertices
            .iter()
            .circular_tuple_windows()
            .map(|(&vl, &v1, &v2, &vr)| {
                let distance = (v2 - v1).length();
                let get_offset = |vl: Vec2, vr: Vec2| {
                    let angle = vl.angle_between(vr);
                    if angle < 0.0 {
                        0.0
                    } else {
                        pixel_to_meter(TILE_HEIGHT - TILE_DOWN * 2.0) / (angle / 2.0).tan()
                    }
                };
                let left_offset = get_offset(vl - v1, v2 - v1);
                let right_offset = get_offset(v1 - v2, vr - v2);

                let distance_offset = distance - left_offset - right_offset;
                let count = (distance_offset / pixel_to_meter(TILE_WIDTH)) as usize;

                let full_left_offset = left_offset
                    + (distance_offset - count as f32 * pixel_to_meter(TILE_WIDTH)) / 2.0;
                let tile_textures = generate_textures_from_tile(assets, tile, v1.x, count)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();

                let rect = add_rect_padding(two_points_rect(v1, v2), pixel_to_meter(TILE_HEIGHT));
                (rect, full_left_offset, tile_textures)
            })
            .collect();
        Self {
            tile,
            colors,
            trimeshes,
            tile_textures,
        }
    }
}

pub fn generate_textures_from_tile<'a>(
    assets: &'a Assets,
    tile: &str,
    seed: f32,
    count: usize,
) -> Vec<&'a str> {
    let tile = &assets.tiles[tile];
    rand::srand(seed.to_bits() as u64);
    let mut last_constraints = TileConstraints::zero();
    let mut textures = Vec::new();
    for i in 0..count {
        let prioritize_decrease = last_constraints.height > (count - i) as u8;
        let possible_textures = tile
            .0
            .iter()
            .filter(|(_, constraints)| last_constraints.fits(constraints.clone()))
            .filter(|(_, constraints)| {
                if prioritize_decrease {
                    constraints.height < last_constraints.height
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();
        let weights = possible_textures
            .iter()
            .map(|(_, constraints)| constraints.weight as f32)
            .collect::<Vec<_>>();
        let mut random_val = rand::gen_range(0.0, weights.iter().sum());
        let index = 'outer: {
            for (i, &weight) in weights.iter().enumerate() {
                random_val -= weight;
                if random_val <= 0.0 {
                    break 'outer i;
                }
            }
            0
        };
        let (texture, constraints) = possible_textures[index];
        last_constraints = constraints.clone();
        textures.push(texture.as_str());
    }
    textures
}

pub fn draw_liquid(liquid_draw: &LiquidDraw) {
    draw_trimesh(
        &liquid_draw.trimesh.0,
        &liquid_draw.trimesh.1,
        liquid_draw.color,
    );
}

pub fn draw_thing(assets: &Assets, world: &World, thing_draw: &ThingDraw, body: &RigidBodyHandle) {
    let body = world.physics_world.get_body(*body).unwrap();
    let pos = body.position().translation.vector.into();
    let rotate = body.position().rotation.angle();
    draw_texture_centered(assets, thing_draw.texture.as_str(), pos, rotate, None);
}
