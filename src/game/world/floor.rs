use super::draw::floor::{LiquidDraw, TiledDraw};
use super::polygon::trimesh_indices_from_polygon;
use super::World;
use crate::consts::*;
use crate::game::texture::TextureHolder;
use macroquad::prelude::*;
use rapier2d::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Grass,
    Stone,
    Water,
    Mud,
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

impl Material {
    pub fn rigid(self) -> bool {
        match self {
            Self::Grass | Self::Stone | Self::Mud => true,
            Self::Water => false,
        }
    }
    pub fn from_hex_color(hex_color: u32) -> Self {
        match hex_color {
            0x50AA59 => Self::Grass,
            0x1E7EB4 => Self::Water,
            0x495380 => Self::Stone,
            0x63403D => Self::Mud,
            _ => panic!("unknown floor color: {:x}", hex_color),
        }
    }
    pub fn to_vertex_draw(self, texture_holder: &TextureHolder, vertices: &[Vec2]) -> VertexDraw {
        match self {
            Self::Grass => VertexDraw::Tiled(TiledDraw::new(
                texture_holder,
                "grass",
                [
                    Color::from_hex(0x8BB661),
                    Color::from_hex(0x50AA59),
                    Color::from_hex(0x449861),
                ],
                vertices,
            )),
            Self::Stone => VertexDraw::Tiled(TiledDraw::new(
                texture_holder,
                "stone",
                [
                    Color::from_hex(0x667696),
                    Color::from_hex(0x54628A),
                    Color::from_hex(0x495380),
                ],
                vertices,
            )),
            Self::Mud => VertexDraw::Tiled(TiledDraw::new(
                texture_holder,
                "mud",
                [
                    Color::from_hex(0x775444),
                    Color::from_hex(0x63403D),
                    Color::from_hex(0x53343C),
                ],
                vertices,
            )),
            Self::Water => VertexDraw::Liquid(LiquidDraw::new(
                vertices,
                with_alpha(Color::from_hex(0x1667B1), 1.0),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VertexDraw {
    Tiled(TiledDraw),
    Liquid(LiquidDraw),
}

pub fn spawn_floor(
    texture_holder: &TextureHolder,
    world: &mut World,
    vertices: Vec<Vec2>,
    material: Material,
) {
    let trimesh_indices = trimesh_indices_from_polygon(&vertices);
    let builder = {
        let vertices = vertices.iter().map(|&v| v.into()).collect::<Vec<_>>();
        ColliderBuilder::trimesh(vertices, trimesh_indices.clone())
            .friction(PLATFORM_FRICTION)
            .friction_combine_rule(CoefficientCombineRule::Max)
    };
    let vertex_draw = material.to_vertex_draw(texture_holder, &vertices);

    let (_, collider_handle) = world.physics_world.add_body(
        RigidBodyBuilder::fixed()
            .translation(Vec2::new(0.0, 0.0).into())
            .build(),
        builder.sensor(!material.rigid()).build(),
    );

    world
        .entities
        .spawn((collider_handle, vertex_draw, material));
}

#[derive(Debug, Clone)]
pub struct ThingDraw {
    pub texture: String,
    pub pos: Vec2,
    pub dims: Vec2,
    pub rotate: f32,
}

pub fn spawn_thing(world: &mut World, pos: Vec2, dims: Vec2, rotate: f32, material: Material) {
    let (_, collider_handle) = world.physics_world.add_body(
        RigidBodyBuilder::fixed()
            .translation(pos.into())
            .rotation(rotate)
            .build(),
        ColliderBuilder::cuboid(dims.x, dims.y)
            .friction(PLATFORM_FRICTION)
            .friction_combine_rule(CoefficientCombineRule::Max)
            .sensor(!material.rigid())
            .build(),
    );
    world.entities.spawn((
        collider_handle,
        ThingDraw {
            texture: "stone".to_owned(),
            pos,
            dims,
            rotate,
        },
        material,
    ));
}
