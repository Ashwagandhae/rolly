use std::f32::consts::PI;

use crate::draw::{draw_texture_centered, pixel_to_meter};
use crate::world::player::{Body, Direction, Polly, Transition};
use crate::world::World;
use macroquad::prelude::*;

pub fn draw(world: &World) {
    let player = world
        .physics_world
        .get_body(world.player.body.any_body_handle())
        .unwrap();
    let pos: Vec2 = (*player.translation()).into();
    let rotation = player.rotation().angle();

    match world.player.rolly_polly_transition {
        Transition::Between { time, .. } => {
            draw_olly(world, pos, rotation, time);
        }
        _ => match world.player.body {
            Body::Rolly(_) => draw_rolly(world, pos, rotation),
            Body::Polly(ref polly) => draw_polly(world, polly, pos, rotation),
        },
    };
}

fn draw_olly(world: &World, pos: Vec2, rotation: f32, time: f32) {
    let parts = &[
        (
            "olly-big-back.png",
            vec2(0.0, 0.0),
            vec2(0.0, -10.0),
            0.0,
            0.0,
        ),
        ("olly-back.png", vec2(0.0, 0.0), vec2(0.0, -20.0), 0.0, 0.0),
        (
            "olly-head.png",
            vec2(45.0, 5.0),
            vec2(30.0, -20.0),
            0.0,
            PI / 2.0,
        ),
        (
            "olly-tail.png",
            vec2(-45.0, 5.0),
            vec2(-30.0, -20.0),
            0.0,
            -PI / 2.0,
        ),
    ];
    for (filename, offset_start, offset_end, rotate_offset_start, rotate_offset_end) in parts {
        let offset = offset_start.lerp(*offset_end, time);
        let offset = pixel_to_meter(offset);
        let rotate_offset = rotate_offset_start * (1.0 - time) + rotate_offset_end * time;

        draw_texture_centered(
            world,
            filename,
            pos + offset,
            rotation + rotate_offset,
            Some(DrawTextureParams {
                pivot: Some(pos),
                ..Default::default()
            }),
        );
    }
}

fn draw_rolly(world: &World, pos: Vec2, rotation: f32) {
    draw_texture_centered(world, "rolly.png", pos, rotation, None);
}

fn draw_polly(world: &World, polly: &Polly, pos: Vec2, rotation: f32) {
    draw_feet(world, polly, pos, rotation);

    draw_texture_centered(
        world,
        match world.player.direction {
            Direction::Right => "polly.png",
            Direction::Left => "polly-flip.png",
        },
        pos,
        rotation,
        None,
    );
}

fn draw_feet(world: &World, polly: &Polly, pos: Vec2, rotation: f32) {
    let draw_foot = |offset: Vec2| {
        draw_texture_centered(
            world,
            "polly-foot.png",
            pos + offset,
            rotation,
            Some(DrawTextureParams {
                pivot: Some(pos),
                ..Default::default()
            }),
        );
    };
    let frame = polly.feet_frame.get();
    for i in 1..5 {
        let x = i as f32 * 20.0 - 60.0 + frame * 20.0;
        let offset = pixel_to_meter(vec2(x, 25.0));
        draw_foot(offset);
    }
    // first foot
    let offset = pixel_to_meter(vec2(
        -55.0 + frame * 15.0,
        20.0 + 5.0 * (frame * 4.0).clamp(0.0, 1.0),
    ));
    draw_foot(offset);
    // last foot
    let offset = pixel_to_meter(vec2(
        40.0 + frame * 15.0,
        20.0 + 5.0 * ((1.0 - frame) * 4.0).clamp(0.0, 1.0),
    ));
    draw_foot(offset);
}
