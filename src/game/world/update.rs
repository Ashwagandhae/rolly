use super::floor::Material;
use super::player::{Body, Polly, Rolly};

use super::World;
use crate::consts::*;
use macroquad::prelude::*;
use nalgebra::UnitComplex;
use rapier2d::prelude::*;

pub fn update(world: &mut World) {
    update_zoom(world);
    update_camera(world);

    world.physics_world.update();

    player_body(world);
    player_water(world);
    player_transition(world);
    match world.player.body {
        Body::Rolly(_) => {}
        Body::Polly(_) => player_polly(world),
    }
}

fn update_camera(world: &mut World) {
    let camera_target: Vec2 = (*world
        .physics_world
        .get_body(world.player.body.any_body_handle())
        .unwrap()
        .translation())
    .into();
    let diff = camera_target - world.camera_target;
    world.camera_target += diff * 0.1;
}

fn update_zoom(world: &mut World) {
    if is_key_down(KeyCode::Equal) {
        world.camera_zoom *= 1.01;
    }
    if is_key_down(KeyCode::Minus) {
        world.camera_zoom *= 0.99;
    }
    if is_key_pressed(KeyCode::Key0) {
        world.camera_zoom = 1.0;
    }
}

use super::player::Direction;
fn player_direction(world: &mut World) {
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

fn player_feet_frame(world: &mut World) {
    let polly = world.player.body.unwrap_polly_mut();
    let body = world.physics_world.get_body(polly.body_handle).unwrap();
    polly.feet_frame -= body.linvel().x * if polly.feet_grounded[1] { 0.1 } else { 0.02 };
}

fn player_body(world: &mut World) {
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
    let polly = world.player.body.unwrap_polly_mut();
    let [left_feet_grounded, center_feet_grounded, right_feet_grounded] = polly.feet_grounded;

    let body = world.physics_world.get_body(polly.body_handle).unwrap();
    let mut linvel = *body.linvel();
    let mut angvel = body.angvel();
    let mut rotation = *body.rotation();
    if is_key_pressed(KeyCode::Up) && center_feet_grounded {
        linvel.y = -PLAYER_VEL_Y;
        angvel = 0.0;
    }
    let movement_state = match (is_key_down(KeyCode::Right), is_key_down(KeyCode::Left)) {
        (true, false) => Some(1.0),
        (false, true) => Some(-1.0),
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
    } else if !left_feet_grounded && !right_feet_grounded {
        angvel = 0.0;
        // rotate towards 0
        rotation = rotation.slerp(&UnitComplex::new(0.), 0.3);
    }
    match (left_feet_grounded, right_feet_grounded) {
        (true, false) => {
            angvel += 2.0;
        }
        (false, true) => {
            angvel -= 2.0;
        }
        (false, false) => {}
        _ => (),
    }

    let body = world.physics_world.get_body_mut(polly.body_handle).unwrap();
    body.set_linvel(linvel, true);
    body.set_angvel(angvel, true);
    body.set_rotation(rotation, true);
}

fn player_feet_grounded(world: &mut World) {
    let polly = world.player.body.unwrap_polly();
    let get = |index: usize| {
        for (_, (entity_collider, _)) in world
            .entities
            .query::<(&ColliderHandle, &Material)>()
            .into_iter()
            .filter(|(_, (_, material))| material.rigid())
        {
            if collider_intersecting(&world, *entity_collider, polly.feet_sensor_handles[index]) {
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
    world: &World,
    handle_1: ColliderHandle,
    handle_2: ColliderHandle,
) -> bool {
    world
        .physics_world
        .narrow_phase
        .intersection_pair(handle_1, handle_2)
        == Some(true)
}

fn player_water(world: &mut World) {
    let player_collider = world.player.body.any_collider_handle();
    let in_water = 'outer: {
        for (_, (collider_handle, _)) in world
            .entities
            .query::<(&ColliderHandle, &Material)>()
            .into_iter()
            .filter(|(_, (_, material))| matches! {material, Material::Water})
        {
            if collider_intersecting(&world, *collider_handle, player_collider) {
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
    }
    player_body.set_linvel(linvel, true);
}
