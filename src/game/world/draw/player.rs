use std::f32::consts::PI;

use super::super::player::{Body, Polly, Transition};
use super::lerp;
use super::{draw_texture_centered, pixel_to_meter};
use crate::game::assets::Assets;
use crate::game::world::World;
use macroquad::prelude::*;

pub fn draw(assets: &Assets, world: &World) {
    let player = world
        .physics_world
        .get_body(world.player.body.any_body_handle())
        .unwrap();
    let pos: Vec2 = (*player.translation()).into();
    let rotation = player.rotation().angle();

    match world.player.rolly_polly_transition {
        Transition::Between { time, .. } => {
            draw_olly(assets, world, pos, rotation, time);
        }
        _ => match world.player.body {
            Body::Rolly(_) => draw_rolly(assets, world, pos, rotation),
            Body::Polly(ref polly) => draw_polly(assets, world, polly, pos, rotation),
        },
    };
}

fn draw_olly(assets: &Assets, world: &World, pos: Vec2, rotation: f32, time: f32) {
    let eye_x = world.player.eye_x.get();
    let parts = &[
        ("olly-big-back", vec2(0.0, 0.0), vec2(0.0, -10.0), 0.0, 0.0),
        ("olly-back", vec2(0.0, 0.0), vec2(0.0, -20.0), 0.0, 0.0),
        (
            "olly-head",
            vec2(45.0, 5.0),
            vec2(30.0, -20.0),
            0.0,
            PI / 2.0,
        ),
        (
            "olly-tail",
            vec2(-45.0, 5.0),
            vec2(-30.0, -20.0),
            0.0,
            -PI / 2.0,
        ),
        (
            "olly-eye",
            vec2(50.0 * eye_x, 15.0),
            vec2(35.0 * eye_x, -10.0),
            0.0,
            PI / 2.0 * eye_x,
        ),
    ];
    let eased = simple_easing::quart_in(time);
    for (filename, offset_start, offset_end, rotate_offset_start, rotate_offset_end) in parts {
        let offset = lerp(*offset_start, *offset_end, eased);
        let offset = pixel_to_meter(offset);
        let rotate_offset = lerp(*rotate_offset_start, *rotate_offset_end, eased);

        draw_texture_centered(
            assets,
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

fn draw_polly(assets: &Assets, world: &World, polly: &Polly, pos: Vec2, rotation: f32) {
    draw_feet(assets, world, polly, pos, rotation);

    let velocity = world
        .physics_world
        .get_body(world.player.body.any_body_handle())
        .unwrap()
        .linvel()
        .x;

    let rotation = if polly.feet_grounded[1] {
        rotation + (velocity * 0.03).clamp(-0.3, 0.3)
    } else {
        rotation
    };

    draw_texture_centered(assets, "polly", pos, rotation, None);
    draw_texture_centered(
        assets,
        "olly-eye",
        pos + pixel_to_meter(vec2(50.0 * world.player.eye_x.get(), 15.0)),
        rotation,
        Some(DrawTextureParams {
            pivot: Some(pos),
            ..Default::default()
        }),
    );
}

fn draw_rolly(assets: &Assets, world: &World, pos: Vec2, rotation: f32) {
    draw_texture_centered(assets, "rolly", pos, rotation, None);
    draw_texture_centered(
        assets,
        "olly-eye",
        pos + pixel_to_meter(vec2(15.0 * world.player.eye_x.get(), 35.0)),
        rotation,
        Some(DrawTextureParams {
            pivot: Some(pos),
            ..Default::default()
        }),
    );
}

fn draw_feet(assets: &Assets, _world: &World, polly: &Polly, pos: Vec2, rotation: f32) {
    let draw_foot = |offset: Vec2| {
        draw_texture_centered(
            assets,
            "polly-foot",
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
