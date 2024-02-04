use crate::{
    consts::*,
    game::{assets::Assets, Settings},
};

use super::{life_state::LifeState, World};
use macroquad::prelude::*;

pub mod floor;
pub mod player;

pub fn draw(settings: &Settings, assets: &Assets, world: &World) {
    draw_back(settings, assets, world);
    player::draw(assets, world);
    floor::draw(assets, world);

    draw_life_state(world);
}

pub fn draw_texture_centered(
    assets: &Assets,
    texture_file: &str,
    pos: Vec2,
    rotation: f32,
    params: Option<DrawTextureParams>,
) {
    let (size, texture) = &assets[texture_file];
    let size = Vec2::new(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32));
    draw_texture_ex(
        *texture,
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

pub fn get_camera_zoom(world: &World) -> Vec2 {
    vec2(
        world.camera.zoom.x,
        world.camera.zoom.y * -1.0, // flip y axis for macroquad bug
    )
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

pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: std::ops::Add<Output = T>,
    T: std::ops::Mul<f32, Output = T>,
{
    a * (1.0 - t) + b * t
}

#[allow(dead_code)]
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

use super::polygon::triangulate_polygon;
#[allow(dead_code)]
fn draw_irregular_polygon(vertices: &[Vec2], color: Color) {
    for [v1, v2, v3] in triangulate_polygon(vertices) {
        draw_triangle(v1, v2, v3, color);
    }
}

fn draw_trimesh(vertices: &[Vec2], indices: &[[u32; 3]], color: Color) {
    for [v1, v2, v3] in indices {
        draw_triangle(
            vertices[*v1 as usize],
            vertices[*v2 as usize],
            vertices[*v3 as usize],
            color,
        );
    }
}

fn draw_back(_settings: &Settings, assets: &Assets, world: &World) {
    let back_items: &[((Option<&str>, _, Option<&str>), f32)] = &[
        ((Some("sky_up"), "sky", None), 0.2),
        ((None, "hills", Some("hills_down")), 0.4),
    ];
    for &((up_texture, texture, down_texture), zoom) in back_items {
        let size = {
            let size = assets[texture].0;
            vec2(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32))
        };
        let y = parallax(zoom, 3.0, world.camera.target.y);
        for x in tiled_parallax_x(world, zoom, size.x, 0.0) {
            let pos = Vec2::new(x, y);
            draw_texture_centered(assets, texture, pos, 0.0, None);
        }
        if let Some(up_texture) = up_texture {
            let size_up = {
                let size = assets[up_texture].0;
                vec2(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32))
            };
            for y_up in tiled_parallax_y(world, zoom, size_up.y, 0.0) {
                if y_up + size_up.y * 2.1 >= y {
                    break;
                }
                for x_up in tiled_parallax_x(world, zoom, size_up.x, 0.0) {
                    let pos = Vec2::new(x_up, y_up);
                    draw_texture_centered(assets, up_texture, pos, 0.0, None);
                }
            }
        }
        if let Some(down_texture) = down_texture {
            let size_down = {
                let size = assets[down_texture].0;
                vec2(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32))
            };
            for y_down in tiled_parallax_y(world, zoom, size_down.y, -size_down.y / 2.0).rev() {
                if y_down - size_down.y * 2.0 <= y {
                    break;
                }
                for x_down in tiled_parallax_x(world, zoom, size_down.x, 0.0) {
                    let pos = Vec2::new(x_down, y_down);
                    draw_texture_centered(assets, down_texture, pos, 0.0, None);
                }
            }
        }
    }
}

// fn draw_sky(world: &World) {
//     // don't ask how this works
//     let zoom = 0.1;
//     let camera_start_x = world.camera.target.x - 1.0 / world.camera.zoom.x;
//     let camera_end_x = world.camera.target.x + 1.0 / world.camera.zoom.x;
//     for x in tiled_parallax(zoom, pixel_to_meter(1600.0), camera_start_x, camera_end_x) {
//         let pos = Vec2::new(x, world.camera.target.y);
//         draw_texture_centered(world, "sky", pos, 0.0, None);
//     }
// }

fn tiled_parallax_x(
    world: &World,
    zoom: f32,
    size: f32,
    offset_backwards: f32,
) -> impl Iterator<Item = f32> + DoubleEndedIterator {
    let camera_start_x = world.camera.target.x - 1.0 / get_camera_zoom(world).x;
    let camera_end_x = world.camera.target.x + 1.0 / get_camera_zoom(world).x;
    tiled_parallax(zoom, size, offset_backwards, camera_start_x, camera_end_x)
}
fn tiled_parallax_y(
    world: &World,
    zoom: f32,
    size: f32,
    offset_backwards: f32,
) -> impl Iterator<Item = f32> + DoubleEndedIterator {
    let factor = screen_height() / screen_width() * 2.0;
    let camera_start_y = world.camera.target.y - factor / get_camera_zoom(world).y;
    let camera_end_y = world.camera.target.y + factor / get_camera_zoom(world).y;
    tiled_parallax(zoom, size, offset_backwards, camera_start_y, camera_end_y)
}

/// Returns an iterator over the positions of the tiles that should be drawn for a parallax effect
fn tiled_parallax(
    zoom: f32,
    size: f32,
    offset_backwards: f32,
    camera_start: f32,
    camera_end: f32,
) -> impl Iterator<Item = f32> + DoubleEndedIterator {
    let camera_target = (camera_start + camera_end) / 2.0;
    let start = (((camera_start - (camera_end - camera_start) * (1.0 - zoom) / zoom * 0.5) / size
        * zoom)
        .floor())
        * size
        + camera_target * (1.0 - zoom)
        - offset_backwards;
    let count = ((camera_end - start) / size).ceil() as usize;
    (0..count).map(move |i| start + size * i as f32 + size / 2.0)
}

fn parallax(zoom: f32, pos: f32, camera_target: f32) -> f32 {
    pos + (camera_target - pos) * (1.0 - zoom)
}

pub fn get_camera_rect(world: &World) -> Rect {
    let camera_zoom = get_camera_zoom(world);
    let camera_target = world.camera.target;
    let camera_start = camera_target - vec2(1.0, screen_height() / screen_height()) / camera_zoom;
    let camera_end = camera_target + vec2(1.0, screen_height() / screen_height()) / camera_zoom;
    Rect::new(
        camera_start.x,
        camera_start.y,
        camera_end.x - camera_start.x,
        camera_end.y - camera_start.y,
    )
}

pub fn pos_in_camera(world: &World, pos: Vec2) -> bool {
    get_camera_rect(world).contains(pos)
}

fn draw_life_state(world: &World) {
    set_default_camera();
    let darkness = match &world.player.life_state {
        LifeState::Alive(t) => t.get(),
        LifeState::Dead(t) => 1.0 - t.get(),
    };
    if darkness <= 0.0 {
        return;
    }
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, darkness),
    );
}
