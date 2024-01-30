use std::f32::consts::PI;

use super::super::floor::{ThingDraw, VertexDraw};
use super::super::polygon::{shrink_polygon, trimesh_indices_from_polygon};
use super::{draw_texture_centered, draw_trimesh, pixel_to_meter};
use crate::consts::*;
use crate::game::texture::{TextureHolder, TileConstraints};
use crate::game::world::World;
use itertools::Itertools;
use macroquad::prelude::*;

pub fn draw(texture_holder: &TextureHolder, world: &World) {
    for (_, draw) in world.entities.query::<&VertexDraw>().iter() {
        match draw {
            VertexDraw::Tiled(tiled_draw) => {
                draw_tiled(texture_holder, tiled_draw);
            }
            VertexDraw::Liquid(liquid_draw) => {
                draw_liquid(liquid_draw);
            }
        }
    }
    for (_, draw) in world.entities.query::<&ThingDraw>().iter() {
        draw_thing(texture_holder, draw);
    }
}

pub fn draw_tiled(texture_holder: &TextureHolder, tiled_draw: &TiledDraw) {
    for ((vertices, indices), color) in tiled_draw.trimeshes.iter().zip(tiled_draw.colors.iter()) {
        draw_trimesh(vertices, indices, *color)
    }
    let vertices = &tiled_draw.trimeshes[0].0;
    for ((_, &v1, &v2, _), (left_offset, textures)) in vertices
        .iter()
        .circular_tuple_windows()
        .zip(tiled_draw.tile_textures.iter())
    {
        let rotation = Vec2::new(1.0, 0.0).angle_between(v1 - v2);
        let rotation_down = rotation + PI / 2.0;
        let draw_on_line = |dist: f32, texture_file: &str| {
            let down = Vec2::from_angle(rotation_down) * pixel_to_meter(TILE_DOWN);
            let pos = v1 + (v2 - v1).normalize() * dist + down;

            draw_texture_centered(texture_holder, texture_file, pos, rotation, None);
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
    pub tile_textures: Vec<(f32, Vec<String>)>,
}

impl TiledDraw {
    pub fn new(
        texture_holder: &TextureHolder,
        tile: &'static str,
        colors: [Color; 3],
        vertices: &[Vec2],
    ) -> Self {
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
                let tile_textures = generate_textures_from_tile(texture_holder, tile, v1.x, count)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                (full_left_offset, tile_textures)
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
    texture_holder: &'a TextureHolder,
    tile: &str,
    seed: f32,
    count: usize,
) -> Vec<&'a str> {
    let tile = &texture_holder.tiles[tile];
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

pub fn draw_thing(texture_holder: &TextureHolder, thing_draw: &ThingDraw) {
    let pos = thing_draw.pos;
    let rotate = thing_draw.rotate;
    draw_texture_centered(
        texture_holder,
        thing_draw.texture.as_str(),
        pos,
        rotate,
        None,
    );
}
