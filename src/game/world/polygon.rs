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
    let map = |v: Vec2| -> Option<u32> { vertices_index_map.get(&v.into()).copied() };
    let indices = triangles
        .into_iter()
        .filter_map(|[v1, v2, v3]| Some([map(v1)?, map(v2)?, map(v3)?]))
        .collect::<Vec<_>>();
    indices
}

// only include outer edge triangles
pub fn trimesh_indices_from_polygon_minimal(vertices: &[Vec2]) -> Vec<[u32; 3]> {
    let vertices_index_map: HashMap<OrderedVec2, u32> = vertices
        .iter()
        .enumerate()
        .map(|(i, v)| ((*v).into(), i as u32))
        .collect();
    let triangles = triangulate_polygon(vertices);
    let map = |v: Vec2| -> u32 { vertices_index_map[&v.into()] };
    let index_distance = |i: u32, j: u32| -> u32 {
        // get circular distance between two indices
        let len = vertices.len() as u32;
        let distance = (i as i32 - j as i32).abs() as u32;
        if distance > len / 2 {
            len - distance
        } else {
            distance
        }
    };
    let indices = triangles
        .into_iter()
        .filter_map(|[v1, v2, v3]| {
            let ret = [map(v1), map(v2), map(v3)];
            if index_distance(ret[0], ret[1]) == 1
                || index_distance(ret[1], ret[2]) == 1
                || index_distance(ret[2], ret[0]) == 1
            {
                Some(ret)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    indices
}

/// When wanting to draw a rect aligned with and inside the edges of a polygon, they can intersect.
/// This function adds padding to the rect to avoid that.
pub fn get_rect_offset_under_polygon_edge(vl: Vec2, vr: Vec2, height: f32) -> f32 {
    let angle = vl.angle_between(vr);
    if angle < 0.0 {
        0.0
    } else {
        height / (angle / 2.0).tan()
    }
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

pub fn two_points_rect(v1: Vec2, v2: Vec2) -> Rect {
    let start = vec2(v1.x.min(v2.x), v1.y.min(v2.y));
    let end = vec2(v1.x.max(v2.x), v1.y.max(v2.y));
    Rect::new(start.x, start.y, end.x - start.x, end.y - start.y)
}

pub fn add_rect_padding(rect: Rect, padding: f32) -> Rect {
    Rect::new(
        rect.x - padding,
        rect.y - padding,
        rect.w + padding * 2.0,
        rect.h + padding * 2.0,
    )
}
