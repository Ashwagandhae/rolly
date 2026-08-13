use crate::{
    consts::*,
    game::{Settings, assets::Assets, world::level::LevelId},
};

use super::{World, life_state::LifeState, light::LightGroup, polygon::three_points_rect};
use macroquad::prelude::*;
use rapier2d::dynamics::RigidBodyHandle;

pub mod floor;
pub mod player;

pub fn draw(settings: &Settings, assets: &Assets, world: &World) {
    set_camera(&world.camera);
    draw_back(settings, assets, world);
    player::draw(assets, world);
    floor::draw(assets, world);
    draw_light(world);

    draw_life_state(world);
}

pub fn draw_texture_centered(
    assets: &Assets,
    texture_file: &str,
    pos: Vec2,
    rotation: f32,
    params: Option<DrawTextureParams>,
) {
    draw_texture_centered_with_color(assets, texture_file, pos, rotation, WHITE, params);
}
pub fn draw_texture_centered_with_color(
    assets: &Assets,
    texture_file: &str,
    pos: Vec2,
    rotation: f32,
    color: Color,
    params: Option<DrawTextureParams>,
) {
    let (size, texture) = &assets[texture_file];
    let size = Vec2::new(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32));
    draw_texture_ex(
        *texture,
        pos.x - size.x / 2.0,
        pos.y - size.y / 2.0,
        color,
        DrawTextureParams {
            dest_size: Some(size),
            rotation,
            ..params.unwrap_or_default()
        },
    );
}

pub fn draw_texture_centered_lazy(
    world: &World,
    assets: &Assets,
    texture_file: &str,
    pos: Vec2,
    rotation: f32,
    params: Option<DrawTextureParams>,
) -> bool {
    let (size, _) = &assets[texture_file];
    let size =
        Vec2::new(pixel_to_meter(size.0 as f32), pixel_to_meter(size.1 as f32)) * 2.0f32.sqrt();
    let rect = Rect::new(pos.x - size.x / 2.0, pos.y - size.y / 2.0, size.x, size.y);
    if !get_camera_rect(world).overlaps(&rect) {
        return false;
    }
    draw_texture_centered(assets, texture_file, pos, rotation, params);
    true
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

fn draw_trimesh_lazy(world: &World, vertices: &[Vec2], indices: &[[u32; 3]], color: Color) {
    for [v1, v2, v3] in indices {
        let [v1, v2, v3] = [
            vertices[*v1 as usize],
            vertices[*v2 as usize],
            vertices[*v3 as usize],
        ];
        if !get_camera_rect(world).overlaps(&three_points_rect(v1, v2, v3)) {
            continue;
        }
        draw_triangle(v1, v2, v3, color);
    }
}

fn draw_back(_settings: &Settings, assets: &Assets, world: &World) {
    let back_items: &[((Option<&str>, _, Option<&str>), f32)] = &[
        ((Some("sky_up"), "sky", None), 0.2),
        ((None, "hills", Some("hills_down")), 0.4),
    ];
    let level_texture = |level: LevelId, texture_postfix: &str| -> String {
        format!("{}_{}", level.0, texture_postfix)
    };
    let (previous, target_alpha, target) = world.back.render();
    let level_alphas = if target_alpha == 1.0 {
        vec![(target, 1.0)]
    } else {
        vec![(previous, 1.0), (target, target_alpha)]
    };
    for (level, alpha) in &level_alphas {
        for &((up_postfix, middle_postfix, down_postfix), zoom) in back_items {
            let color = Color::new(1.0, 1.0, 1.0, *alpha);
            let texture = level_texture(*level, middle_postfix);
            let size = assets.texture_size(&texture).unwrap();
            let y = parallax(zoom, 3.0, world.camera.target.y);
            for x in tiled_parallax_x(world, zoom, size.x, 0.0) {
                let pos = Vec2::new(x, y);
                draw_texture_centered_with_color(assets, &texture, pos, 0.0, color, None);
            }
            if let Some(up_texture) = up_postfix {
                let up_texture = level_texture(*level, up_texture);
                let size_up = assets.texture_size(&up_texture).unwrap();
                for y_up in tiled_parallax_y(world, zoom, size_up.y, 0.0) {
                    if y_up + size_up.y * 2.1 >= y {
                        break;
                    }
                    for x_up in tiled_parallax_x(world, zoom, size_up.x, 0.0) {
                        let pos = Vec2::new(x_up, y_up);
                        draw_texture_centered_with_color(
                            assets,
                            &up_texture,
                            pos,
                            0.0,
                            color,
                            None,
                        );
                    }
                }
            }
            if let Some(down_texture) = down_postfix {
                let down_texture = level_texture(*level, down_texture);
                let size_down = assets.texture_size(&down_texture).unwrap();
                for y_down in tiled_parallax_y(world, zoom, size_down.y, -size_down.y / 2.0).rev() {
                    if y_down - size_down.y * 2.0 <= y {
                        break;
                    }
                    for x_down in tiled_parallax_x(world, zoom, size_down.x, 0.0) {
                        let pos = Vec2::new(x_down, y_down);
                        draw_texture_centered_with_color(
                            assets,
                            &down_texture,
                            pos,
                            0.0,
                            color,
                            None,
                        );
                    }
                }
            }
        }
    }
}

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
    // add_rect_padding(
    Rect::new(
        camera_start.x,
        camera_start.y,
        camera_end.x - camera_start.x,
        camera_end.y - camera_start.y,
    )
    //     ,-0.5,
    // )
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

fn draw_light(world: &World) {
    for (_, (body, light)) in world
        .entities
        .query::<(&RigidBodyHandle, &LightGroup)>()
        .iter()
    {
        let body = world.physics_world.get_body(*body).unwrap();
        let pos = Vec2::from(body.position().translation.vector);
        let angle = body.position().rotation.angle();
        for (light, light_state) in &light.lights {
            let pos = pos + light.pos.rotate(Vec2::from_angle(angle));
            let rect = Rect::new(
                pos.x - light.radius,
                pos.y - light.radius,
                light.radius * 2.0,
                light.radius * 2.0,
            );
            if !get_camera_rect(world).overlaps(&rect) {
                continue;
            }
            let color = Color::new(1.0, 1.0, 1.0, light_state.strength());
            draw_circle(pos.x, pos.y, light.radius, color);
        }
    }
}
