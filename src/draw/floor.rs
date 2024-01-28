use std::f32::consts::PI;

use super::{draw_texture_centered, draw_trimesh, pixel_to_meter};
use crate::consts::*;
use crate::polygon::trimesh_indices_from_polygon;
use crate::world::floor::{ThingDraw, VertexDraw};
use crate::world::texture::TileConstraints;
use crate::{polygon::shrink_polygon, world::World};
use itertools::Itertools;
use macroquad::prelude::*;
use macroquad::rand::{gen_range, srand};

pub fn draw(world: &World) {
    // for entity in &world.entities {
    //     match entity {
    //         Entity::Floor(floor) => match floor.info.draw {
    //             FloorDraw::Tiled(ref tile_draw) => {
    //                 draw_tiled(world, floor, tile_draw);
    //             }
    //             FloorDraw::Liquid(color) => {
    //                 draw_liquid(world, floor, color);
    //             }
    //         },
    //         Entity::Thing(thing) => {}
    //     }
    // }
    for (_, draw) in world.entities.query::<&VertexDraw>().iter() {
        match draw {
            VertexDraw::Tiled(tile_draw) => {
                draw_tiled(world, tile_draw);
            }
            VertexDraw::Liquid(liquid_draw) => {
                draw_liquid(world, liquid_draw);
            }
        }
    }
    for (_, draw) in world.entities.query::<&ThingDraw>().iter() {
        draw_thing(world, draw);
    }
}

pub fn draw_tiled(world: &World, tiled_draw: &TiledDraw) {
    tiled_draw.draw(world);
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
    pub fn new(world: &World, tile: &'static str, colors: [Color; 3], vertices: &[Vec2]) -> Self {
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
                let tile_textures = generate_textures_from_tile(world, tile, v1.x, count)
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
    pub fn draw(&self, world: &World) {
        for ((vertices, indices), color) in self.trimeshes.iter().zip(self.colors.iter()) {
            draw_trimesh(vertices, indices, *color)
        }
        let vertices = &self.trimeshes[0].0;
        for ((_, &v1, &v2, _), (left_offset, textures)) in vertices
            .iter()
            .circular_tuple_windows()
            .zip(self.tile_textures.iter())
        {
            let rotation = Vec2::new(1.0, 0.0).angle_between(v1 - v2);
            let rotation_down = rotation + PI / 2.0;
            let draw_on_line = |dist: f32, texture_file: &str| {
                let down = Vec2::from_angle(rotation_down) * pixel_to_meter(TILE_DOWN);
                let pos = v1 + (v2 - v1).normalize() * dist + down;

                draw_texture_centered(world, texture_file, pos, rotation, None);
            };
            for (j, texture_file) in textures.iter().rev().enumerate() {
                let dist = j as f32 * pixel_to_meter(TILE_WIDTH)
                    + pixel_to_meter(TILE_WIDTH / 2.0)
                    + left_offset;
                draw_on_line(dist, texture_file);
            }
        }
    }
}

pub fn generate_textures_from_tile<'a>(
    world: &'a World,
    tile: &str,
    seed: f32,
    count: usize,
) -> Vec<&'a str> {
    let tile = &world.texture_holder.tiles[tile];
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

pub fn draw_liquid(world: &World, liquid_draw: &LiquidDraw) {
    // let pos: Vec2 = (*world
    //     .physics_world
    //     .get_collider(floor.collider_handle)
    //     .unwrap()
    //     .translation())
    // .into();
    // let vertices = floor
    //     .vertices
    //     .iter()
    //     .map(|v| *v + pos)
    //     .collect::<Vec<Vec2>>();
    draw_trimesh(
        &liquid_draw.trimesh.0,
        &liquid_draw.trimesh.1,
        liquid_draw.color,
    );
}

pub fn draw_thing(world: &World, thing_draw: &ThingDraw) {
    let pos = thing_draw.pos;
    let rotate = thing_draw.rotate;
    draw_texture_centered(world, thing_draw.texture.as_str(), pos, rotate, None);
}
