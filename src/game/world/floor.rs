use super::draw::floor::{LiquidDraw, TiledDraw};
use super::draw::pixel_to_meter;
use super::level::LevelId;
use super::polygon::{
    add_rect_padding, get_rect_offset_under_polygon_edge, trimesh_from_polygon, two_points_rect,
};
use super::World;
use crate::consts::*;
use crate::game::assets::Assets;
use itertools::Itertools;
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
    pub fn to_vertex_draw(self, assets: &Assets, vertices: &[Vec2]) -> VertexDraw {
        match self {
            Self::Grass => VertexDraw::Tiled(TiledDraw::new(
                assets,
                "grass",
                [
                    Color::from_hex(0x8BB661),
                    Color::from_hex(0x50AA59),
                    Color::from_hex(0x449861),
                ],
                vertices,
            )),
            Self::Stone => VertexDraw::Tiled(TiledDraw::new(
                assets,
                "stone",
                [
                    Color::from_hex(0x667696),
                    Color::from_hex(0x54628A),
                    Color::from_hex(0x495380),
                ],
                vertices,
            )),
            Self::Mud => VertexDraw::Tiled(TiledDraw::new(
                assets,
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
                with_alpha(Color::from_hex(0x1667B1), 0.7),
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
    assets: &Assets,
    world: &mut World,
    vertices: Vec<Vec2>,
    material: Material,
    level: LevelId,
    pos: Vec2,
) {
    let vertices = vertices.iter().map(|v| *v + pos).collect::<Vec<_>>();

    let vertex_draw = material.to_vertex_draw(assets, &vertices);

    let body_handle = world.physics_world.add_body(
        RigidBodyBuilder::fixed()
            .translation(Vec2::ZERO.into())
            .build(),
    );
    world
        .entities
        .spawn((body_handle, vertex_draw, material, level));

    if material.rigid() {
        for (rect, builder) in polygon_colliders_from_rects(&vertices) {
            let builder = builder
                .friction(PLATFORM_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Max);
            world.entities.spawn((
                LazyCollider {
                    rect,
                    builder,
                    body_handle,
                },
                material,
                level,
            ));
        }
    } else {
        let indices = trimesh_from_polygon(&vertices);
        let vertices = vertices.iter().map(|&v| v.into()).collect::<Vec<_>>();
        let builder = ColliderBuilder::trimesh(vertices, indices).sensor(true);
        let handle = world.physics_world.add_collider(
            builder
                .friction(PLATFORM_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Max)
                .build(),
            body_handle,
        );
        world.entities.spawn((handle, material, level));
    }
}

fn polygon_colliders_from_rects(vertices: &[Vec2]) -> Vec<(Rect, ColliderBuilder)> {
    vertices
        .iter()
        .circular_tuple_windows()
        .map(|(&vl, &v1, &v2, &vr)| {
            let distance = (v2 - v1).length();
            let height = pixel_to_meter(10.0);
            let left_offset = get_rect_offset_under_polygon_edge(vl - v1, v2 - v1, height);
            let right_offset = get_rect_offset_under_polygon_edge(v1 - v2, vr - v2, height);
            let distance_offset = distance - left_offset - right_offset;
            let rotation = Vec2::new(1.0, 0.0).angle_between(v1 - v2);
            let rotation_down = rotation + std::f32::consts::PI / 2.0;
            let pos = v1
                + (v2 - v1).normalize() * (left_offset + distance_offset / 2.0)
                + Vec2::from_angle(rotation_down) * height / 2.0;
            let builder = ColliderBuilder::cuboid(distance_offset / 2.0, height / 2.0)
                .translation(pos.into())
                .rotation(rotation);
            let rect = add_rect_padding(two_points_rect(v1, v2), pixel_to_meter(30.0));
            (rect, builder)
        })
        .collect()
}

pub struct LazyCollider {
    pub rect: Rect,
    pub builder: ColliderBuilder,
    pub body_handle: RigidBodyHandle,
}
