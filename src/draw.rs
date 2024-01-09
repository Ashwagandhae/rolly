use crate::consts::PIXEL_TO_METER;

use super::world::World;
use macroquad::prelude::*;

pub mod player;

pub fn draw(world: &World) {
    set_default_camera();
    clear_background(BLACK);
    draw_text("IT WORKS!", 20.0, 20.0, 30.0, WHITE);

    set_camera(&world.camera);
    draw_floors(world);
    player::draw(world);
}

pub fn draw_texture_centered(
    world: &World,
    texture_file: &str,
    pos: Vec2,
    rotation: f32,
    params: Option<DrawTextureParams>,
) {
    let (size, texture) = &world.texture_holder[texture_file];
    let size = Vec2::new(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32));
    draw_texture_ex(
        texture,
        pos.x - size.x / 2.0,
        pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation,
            ..params.unwrap_or_default()
        },
    );
}

pub fn pixel_to_meter<T>(pixel: T) -> T
where
    T: std::ops::Div<f32, Output = T>,
{
    pixel / PIXEL_TO_METER
}

pub fn meter_to_pixel<T>(meter: T) -> T
where
    T: std::ops::Mul<f32, Output = T>,
{
    meter * PIXEL_TO_METER
}

fn draw_floors(world: &World) {
    for floor in &world.floors {
        let pos: Vec2 = (*world
            .physics_world
            .get_body(floor.body_handle)
            .unwrap()
            .translation())
        .into();
        let vertices = floor
            .vertices
            .iter()
            .map(|v| *v + pos)
            .collect::<Vec<Vec2>>();
        match vertices.len() {
            0..=2 => panic!("too few vertices: {}", vertices.len()),
            3 => {
                draw_triangle(vertices[0], vertices[1], vertices[2], BLUE);
            }
            _ => draw_irregular_polygon(&vertices, BLUE),
        }
    }
}

fn draw_irregular_polygon_lines(vertices: &[Vec2], thickness: f32, color: Color) {
    let mut vertices = vertices.to_vec();
    vertices.push(vertices[0]);
    for i in 0..vertices.len() - 1 {
        draw_line(
            vertices[i].x,
            vertices[i].y,
            vertices[i + 1].x,
            vertices[i + 1].y,
            thickness,
            color,
        );
    }
}

use poly2tri_rs::{Point, SweeperBuilder};

fn draw_irregular_polygon(vertices: &[Vec2], color: Color) {
    let sweeper = SweeperBuilder::new(
        vertices
            .iter()
            .map(|v| Point::new(v.x.into(), v.y.into()))
            .collect::<Vec<_>>(),
    )
    .build();
    let triangles = sweeper.triangulate();
    for triangle in triangles {
        let [v1, v2, v3] = triangle.points;
        draw_triangle(
            vec2(v1.x as f32, v1.y as f32),
            vec2(v2.x as f32, v2.y as f32),
            vec2(v3.x as f32, v3.y as f32),
            color,
        );
    }
}
