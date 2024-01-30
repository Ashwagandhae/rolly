use std::collections::HashMap;

use itertools::Itertools;
use macroquad::prelude::*;
use ordered_float::OrderedFloat;
use poly2tri_rs::{Point, SweeperBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderedVec2 {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
}

impl OrderedVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
        }
    }
}

impl From<OrderedVec2> for Vec2 {
    fn from(ordered_vec2: OrderedVec2) -> Self {
        Self::new(ordered_vec2.x.into_inner(), ordered_vec2.y.into_inner())
    }
}

impl From<Vec2> for OrderedVec2 {
    fn from(vec2: Vec2) -> Self {
        Self::new(vec2.x, vec2.y)
    }
}

pub fn trimesh_indices_from_polygon(vertices: &[Vec2]) -> Vec<[u32; 3]> {
    let vertices_index_map: HashMap<OrderedVec2, u32> = vertices
        .iter()
        .enumerate()
        .map(|(i, v)| ((*v).into(), i as u32))
        .collect();
    let triangles = triangulate_polygon(vertices);
    let map = |v: Vec2| -> u32 { vertices_index_map[&v.into()] };
    let indices = triangles
        .into_iter()
        .map(|[v1, v2, v3]| [map(v1), map(v2), map(v3)])
        .collect::<Vec<_>>();
    indices
}

pub fn triangulate_polygon(vertices: &[Vec2]) -> Vec<[Vec2; 3]> {
    match vertices.len() {
        0..=2 => panic!("too few vertices: {}", vertices.len()),
        3 => vec![[vertices[0], vertices[1], vertices[2]]],
        _ => triangulate_polygon_over_4(vertices),
    }
}

pub fn triangulate_polygon_over_4(vertices: &[Vec2]) -> Vec<[Vec2; 3]> {
    let sweeper = SweeperBuilder::new(
        vertices
            .iter()
            .map(|v| Point::new(v.x.into(), v.y.into()))
            .collect::<Vec<_>>(),
    )
    .build();
    let triangles = sweeper.triangulate();
    triangles
        .into_iter()
        .map(|triangle| {
            let [v1, v2, v3] = triangle.points;
            [
                vec2(v1.x as f32, v1.y as f32),
                vec2(v2.x as f32, v2.y as f32),
                vec2(v3.x as f32, v3.y as f32),
            ]
        })
        .collect::<Vec<_>>()
}

pub fn shrink_polygon(vertices: &[Vec2], shrink: f32) -> Vec<Vec2> {
    vertices
        .iter()
        .circular_tuple_windows()
        .map(|(&v1, &v2, &v3)| {
            let angle = (v1 - v2).angle_between(v3 - v2) / 2.0;
            let down = Vec2::from_angle(angle).rotate((v1 - v2).normalize())
                * if angle > 0.0 { 1.0 } else { -1.0 };
            let shrink = shrink / angle.sin().abs();
            v2 + down * shrink
        })
        .collect::<Vec<_>>()
}

// see this https://stackoverflow.com/a/1165943
pub fn vertices_to_clockwise(vertices: Vec<Vec2>) -> Vec<Vec2> {
    let mut vertices = vertices;
    let sum: f32 = vertices
        .iter()
        .circular_tuple_windows()
        .map(|(v1, v2)| (v2.x - v1.x) * (v2.y + v1.y))
        .sum();
    if sum < 0.0 {
        vertices.reverse();
    }
    vertices
}
