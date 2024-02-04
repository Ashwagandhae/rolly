use std::f32::consts::PI;

use super::draw::{get_camera_rect, pixel_to_meter};
use super::floor::{LazyCollider, Material};
use super::frame::Transition;
use super::level::{
    load_level, unload_level, update_loaded_levels, update_loaded_levels_alive, LevelId,
};
use super::life_state::LifeState;
use super::physics_world::PhysicsWorld;
use super::player::{Body, Polly, Rolly};

use super::thing::{AreaOfEffect, Respawn};
use super::World;
use crate::consts::*;
use crate::game::assets::Assets;
use crate::game::Settings;
use macroquad::prelude::*;
use nalgebra::UnitComplex;
use rapier2d::prelude::*;

pub fn update(assets: &Assets, settings: &Settings, world: &mut World) {
    update_camera(settings, world);

    update_loaded_levels_alive(assets, world);
    update_lazy_collider(world);

    world.physics_world.update();

    player_body(world);
    player_water(world);
    player_fall(world);

    player_transition(world);

    player_respawn(world);

    update_life_state(assets, world);
    match world.player.body {
        Body::Rolly(_) => {}
        Body::Polly(_) => player_polly(world),
    }
}

fn update_camera(settings: &Settings, world: &mut World) {
    if let LifeState::Alive(Transition::End)
    | LifeState::Dead(Transition::Start | Transition::Between { .. }) = world.player.life_state
    {
        let camera_target: Vec2 = (*world
            .physics_world
            .get_body(world.player.body.any_body_handle())
            .unwrap()
            .translation())
        .into();
        let diff = camera_target - world.camera.target;
        world.camera.target += diff * settings.camera_speed.value;
    }

    world.camera = Camera2D {
        // zoom: camera_zoom(world), — works in macroquad 0.4.*
        // need to use this to make it work in 0.3.*, to address screen flipping bug
        zoom: vec2(camera_zoom(settings).x, -camera_zoom(settings).y),
        ..world.camera
    };
    set_camera(&world.camera);
}

pub fn camera_zoom(settings: &Settings) -> Vec2 {
    vec2(
        1. * ZOOM * settings.zoom.value,
        screen_width() / screen_height() * ZOOM * settings.zoom.value,
    )
}

use super::player::Direction;
fn player_direction(world: &mut World) {
    if let LifeState::Alive(Transition::End) = world.player.life_state {
        match (is_key_down(KeyCode::Right), is_key_down(KeyCode::Left)) {
            (true, false) => {
                world.player.eye_x.set(1.0);
                world.player.direction = Direction::Right;
            }
            (false, true) => {
                world.player.eye_x.set(-1.0);
                world.player.direction = Direction::Left;
            }
            _ => (),
        }
    }
}

fn player_feet_frame(world: &mut World) {
    let polly = world.player.body.unwrap_polly_mut();
    let body = world.physics_world.get_body(polly.body_handle).unwrap();
    polly.feet_frame -= body.linvel().x * if polly.feet_grounded[1] { 0.1 } else { 0.02 };
}

fn player_body(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    if is_key_pressed(KeyCode::Down) {
        let body = world
            .physics_world
            .get_body_mut(world.player.body.any_body_handle())
            .unwrap();
        let translation = *body.translation();
        let rotation = *body.rotation();
        let linvel = *body.linvel();
        let angvel = body.angvel();
        match world.player.body.clone() {
            Body::Polly(polly) => {
                polly.despawn(&mut world.physics_world);
                world.player.rolly_polly_transition.run(0.3, false);
                world.player.body = Body::Rolly(Rolly::spawn(
                    &mut world.physics_world,
                    translation.into(),
                    rotation.angle(),
                    linvel.into(),
                    angvel,
                ))
            }
            Body::Rolly(rolly) => {
                rolly.despawn(&mut world.physics_world);
                world.player.rolly_polly_transition.run(0.3, true);
                world.player.body = Body::Polly(Polly::spawn(
                    &mut world.physics_world,
                    translation.into(),
                    rotation.angle(),
                    linvel.into(),
                    angvel,
                ))
            }
        };
    }
}

fn player_transition(world: &mut World) {
    world.player.rolly_polly_transition.tick(get_frame_time());
    world.player.eye_x.tick(get_frame_time());
}

fn player_polly(world: &mut World) {
    player_feet_grounded(world);
    player_feet_frame(world);
    player_direction(world);
    player_movement(world);
}

fn player_movement(world: &mut World) {
    let alive = if let LifeState::Alive(Transition::End) = world.player.life_state {
        true
    } else {
        false
    };
    let polly = world.player.body.unwrap_polly_mut();
    let [left_feet_grounded, center_feet_grounded, right_feet_grounded] = polly.feet_grounded;

    let body = world.physics_world.get_body(polly.body_handle).unwrap();
    let mut linvel = *body.linvel();
    let mut angvel = body.angvel();
    let mut rotation = *body.rotation();
    if alive && is_key_pressed(KeyCode::Up) && center_feet_grounded {
        linvel.y = -PLAYER_VEL_Y;
        angvel = 0.0;
    }
    let movement_state = match (
        alive,
        is_key_down(KeyCode::Right),
        is_key_down(KeyCode::Left),
    ) {
        (false, _, _) => None,
        (_, true, false) => Some(1.0),
        (_, false, true) => Some(-1.0),
        _ => None,
    };

    if let Some(dir) = movement_state {
        let vel = if center_feet_grounded {
            PLAYER_VEL_X_GROUNDED
        } else {
            PLAYER_VEL_X
        };
        let vel = vel * get_frame_time();
        if linvel.x.abs() < PLAYER_MAX_VEL {
            linvel.x += vel * dir;
        }
        if center_feet_grounded {
            linvel.y -= vel;
        }
    }

    if center_feet_grounded {
        let delta = linvel * 0.8 * get_frame_time();
        linvel -= delta;

        // let delta = angvel * 0.8 * get_frame_time();
        // angvel -= delta;
    } else if alive && !left_feet_grounded && !right_feet_grounded {
        angvel = 0.0;
        // rotate towards 0
        rotation = rotation.slerp(&UnitComplex::new(0.), 0.3);
    }
    if alive {
        match (left_feet_grounded, right_feet_grounded) {
            (true, false) => {
                angvel += 2.0;
            }
            (false, true) => {
                angvel -= 2.0;
            }
            _ => (),
        }
    }

    let body = world.physics_world.get_body_mut(polly.body_handle).unwrap();
    body.set_linvel(linvel, false);
    body.set_angvel(angvel, false);
    body.set_rotation(rotation, false);
}

fn player_feet_grounded(world: &mut World) {
    let polly = world.player.body.unwrap_polly();
    let mut get = |index: usize| {
        for (_, (entity_collider, _)) in world
            .entities
            .query_mut::<(&ColliderHandle, &Material)>()
            .into_iter()
            .filter(|(_, (_, material))| material.rigid())
        {
            if collider_intersecting(
                &world.physics_world,
                *entity_collider,
                polly.feet_sensor_handles[index],
            ) {
                return true;
            }
        }
        false
    };
    let feet_grounded = [get(0), get(1), get(2)];
    let polly = world.player.body.unwrap_polly_mut();
    polly.feet_grounded = feet_grounded;
}

fn collider_intersecting(
    physics_world: &PhysicsWorld,
    handle_1: ColliderHandle,
    handle_2: ColliderHandle,
) -> bool {
    physics_world
        .narrow_phase
        .intersection_pair(handle_1, handle_2)
        == Some(true)
}

fn player_water(world: &mut World) {
    let player_collider = world.player.body.any_collider_handle();
    let in_water = 'outer: {
        for (_, (collider_handle, _)) in world
            .entities
            .query_mut::<(&ColliderHandle, &Material)>()
            .into_iter()
            .filter(|(_, (_, material))| matches! {material, Material::Water})
        {
            if collider_intersecting(&world.physics_world, *collider_handle, player_collider) {
                break 'outer true;
            }
        }
        false
    };
    let player_body = world.player.body.any_body_handle();
    let player_body = world.physics_world.get_body_mut(player_body).unwrap();
    let mut linvel = *player_body.linvel();
    if in_water {
        linvel.x -= linvel.x * (0.5 * get_frame_time()).clamp(0.0, 1.0);
        linvel.y -= linvel.y * (0.5 * get_frame_time()).clamp(0.0, 1.0);
        linvel.y -= 8.0 * get_frame_time();
        if let LifeState::Alive(Transition::End) = world.player.life_state {
            if let Body::Polly(_) = world.player.body {
                world.player.life_state = LifeState::Dead(Transition::Start);
            }
        }
    }
    player_body.set_linvel(linvel, true);
}

fn update_life_state(assets: &Assets, world: &mut World) {
    let (LifeState::Alive(transition) | LifeState::Dead(transition)) = &mut world.player.life_state;
    let old_transition = transition.clone();
    if let Transition::Start = transition {
        transition.run(DEAD_ALIVE_TRANSITION_DURATION, true);
    }
    transition.tick(get_frame_time());
    match (old_transition, world.player.life_state.clone()) {
        (Transition::Between { .. }, LifeState::Alive(Transition::End)) => {
            respawn_player(world);
        }
        (_, LifeState::Dead(Transition::End)) => {
            load_respawn(assets, world);
            world.player.life_state = LifeState::Alive(Transition::Start);
        }
        _ => (),
    }
}

pub fn respawn_player(world: &mut World) {
    world.player.reset(&mut world.physics_world);

    let (pos, rotation) = find_respawn(world, world.player.respawn);
    let angle_up = UnitComplex::from_angle(rotation + PI);

    let pos = Vector::from(pos) + angle_up.transform_vector(&vector![0.0, pixel_to_meter(45.0)]);
    let linvel = angle_up.transform_vector(&vector![0.0, PLAYER_RESPAWN_BOOST]);

    let body = world.player.body.any_body_handle();
    let body = world.physics_world.get_body_mut(body).unwrap();

    body.set_translation(pos, true);
    body.set_linvel(linvel, true);
}

fn load_respawn(assets: &Assets, world: &mut World) {
    for (level, _) in world.levels.clone() {
        unload_level(world, level)
    }
    load_level(assets, world, world.player.respawn, vec2(0.0, 0.0));

    let (pos, _) = find_respawn(world, world.player.respawn);
    world.camera.target = pos;

    update_loaded_levels(assets, world);
}

fn find_respawn(world: &World, respawn: LevelId) -> (Vec2, f32) {
    world
        .entities
        .query::<(&Respawn, &LevelId, &RigidBodyHandle)>()
        .into_iter()
        .filter(|(_, (_, &level, _))| level == respawn)
        .map(|(_, (_, _, pos))| {
            let body = world.physics_world.get_body(*pos).unwrap();
            ((*body.translation()).into(), body.rotation().angle())
        })
        .next()
        .unwrap()
}

fn get_player_body(world: &World) -> &RigidBody {
    let body = world.player.body.any_body_handle();
    world.physics_world.get_body(body).unwrap()
}

fn player_respawn(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    let body = get_player_body(world);
    let player_pos: Vec2 = (*body.translation()).into();
    let current_respawn = world.player.respawn;
    for (_, (_, _, _, level)) in world
        .entities
        .query_mut::<(&Respawn, &RigidBodyHandle, &AreaOfEffect, &LevelId)>()
        .into_iter()
        .filter(|(_, (_, _, _, &level))| level >= current_respawn)
        .filter(|(_, (_, handle, area, _))| area.contains(handle, &world.physics_world, player_pos))
    {
        world.player.respawn = *level;
    }
    if is_key_pressed(KeyCode::R) {
        world.player.life_state = LifeState::Dead(Transition::Start);
    }
}

fn player_fall(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    let body = get_player_body(world);
    let player_pos: Vec2 = (*body.translation()).into();
    if player_pos.y > 10.0 {
        world.player.life_state = LifeState::Dead(Transition::Start);
    }
}

fn update_lazy_collider(world: &mut World) {
    let mut entities_remove_collider = Vec::new();
    let mut entities_add_collider = Vec::new();
    for (entity, (lazy_collider, handle)) in world
        .entities
        .query::<(&mut LazyCollider, Option<&ColliderHandle>)>()
        .iter()
    {
        let player_pos = get_player_body(world).position();
        let player_rect = Rect {
            x: player_pos.translation.vector.x - LAZY_PLAYER_RECT / 2.0,
            y: player_pos.translation.vector.y - LAZY_PLAYER_RECT / 2.0,
            w: LAZY_PLAYER_RECT,
            h: LAZY_PLAYER_RECT,
        };
        let overlaps = player_rect.overlaps(&lazy_collider.rect);
        if let Some(handle) = handle {
            if !overlaps {
                world.physics_world.remove_collider(*handle);
                entities_remove_collider.push(entity);
            }
        } else {
            if overlaps {
                let handle = world.physics_world.add_collider(
                    lazy_collider.builder.clone().build(),
                    lazy_collider.body_handle,
                );
                entities_add_collider.push((entity, handle));
            }
        }
    }
    for entity in entities_remove_collider {
        world.entities.remove_one::<ColliderHandle>(entity).unwrap();
    }
    for (entity, handle) in entities_add_collider {
        world.entities.insert_one(entity, handle).unwrap();
    }
}
