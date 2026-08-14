use std::f32::consts::PI;

use super::draw::pixel_to_meter;
use super::floor::{LazyCollider, Material};
use super::frame::Transition;
use super::level::{
    LevelId, load_level, unload_level, update_loaded_levels, update_loaded_levels_alive,
};
use super::life_state::LifeState;
use super::light::LightGroup;
use super::physics_world::PhysicsWorld;
use super::player::{Body, Polly, Rolly};

use super::World;
use super::thing::{AreaOfEffect, Respawn, ThingId};
use crate::consts::*;
use crate::game::Settings;
use crate::game::assets::Assets;
use crate::game::config::GameConfig;
use crate::game::world::level::load_level_at_pos;
use crate::game::world::light::{LightState, Ripple, RippleSource, RippleState};
use crate::game::world::thing::{Flytrap, Mushroom, RespawnActive, ThingDraw};
use macroquad::prelude::*;
use nalgebra::UnitComplex;
use ordered_float::OrderedFloat;
use rapier2d::prelude::*;

pub async fn update(
    assets: &mut Assets,
    settings: &Settings,
    world: &mut World,
    config: &GameConfig,
) {
    update_camera(settings, world);

    update_loaded_levels_alive(assets, world);
    update_lazy_collider(world);

    world.physics_world.update();

    player_body(world);
    player_water(world);
    player_mushroom(world);
    player_flytrap(world);
    update_ripple_source(world);
    player_fall(world);
    if config.cheat {
        player_cheat_movement(world);
        player_cheat_assets(assets, world).await;
    }

    player_transition(world);
    respawn_transition(world);

    player_respawn(world);

    update_life_state(assets, world);
    update_back(world, assets);
    update_light(world);
    update_ripple(world);
    update_flytrap_teeth(world);

    match world.player.body {
        Body::Rolly(_) => {}
        Body::Polly(_) => player_polly(world),
    }
}
fn update_ripple_source(world: &mut World) {
    let mut ripples = Vec::new();
    let player_pos = Vec2::from(get_player_body(world).position().translation);
    for (_, (handle, source)) in world
        .entities
        .query_mut::<(&RigidBodyHandle, &mut RippleSource)>()
    {
        let body = world.physics_world.rigid_body_set.get(*handle).unwrap();
        let contact = body
            .colliders()
            .iter()
            .flat_map(|collider| {
                world
                    .physics_world
                    .narrow_phase
                    .contact_pairs_with(*collider)
            })
            .find(|pair| pair.has_any_active_contact);
        if !source.just_contacted && contact.is_some() {
            ripples.push(Ripple::new(player_pos));
        }
        source.just_contacted = contact.is_some();
    }
    for ripple in ripples {
        world.entities.spawn((ripple,));
    }
}
fn update_ripple(world: &mut World) {
    let mut remove = Vec::new();
    for (id, ripple) in world.entities.query_mut::<&mut Ripple>() {
        ripple.radius += 0.5 * get_frame_time();
        if ripple.radius >= 1.0 {
            remove.push(id)
        }
    }
    for id in remove {
        world.entities.despawn(id).unwrap();
    }
}
fn update_flytrap_teeth(world: &mut World) {
    for (_, flytrap) in world.entities.query::<&mut Flytrap>().iter() {
        flytrap.teeth_x += get_frame_time() * flytrap.teeth_speed;
        flytrap.teeth_speed =
            (flytrap.teeth_speed - 1.5 * get_frame_time()).max(FLYTRAP_TEETH_SPEED);
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
fn respawn_transition(world: &mut World) {
    for (_, (respawn, draw, light_group)) in
        world
            .entities
            .query_mut::<(&mut Respawn, &mut ThingDraw, &mut LightGroup)>()
    {
        if let RespawnActive::Active(transition) = &mut respawn.active {
            if transition.get() <= 0.0 {
                continue;
            }
            transition.tick(get_frame_time());
            let new_offset = respawn.offset * simple_easing::quart_in(transition.get());
            draw.offset = new_offset;
            if transition.get() <= 0.0 {
                for (_, state) in light_group.lights.iter_mut() {
                    *state = LightState::Ripple(RippleState {
                        strength: state.strength(),
                    });
                }
            }
        }
    }
}

fn player_polly(world: &mut World) {
    player_feet_grounded(world);
    player_feet_frame(world);
    player_direction(world);
    player_movement(world);
}
const CHEAT_MOVE_SPEED: f32 = 0.1;
fn player_cheat_movement(world: &mut World) {
    let handle = world.player.body.any_body_handle();
    let rigid_body = world.physics_world.get_body_mut(handle).unwrap();
    let mut pos = *rigid_body.position();
    let mut used = false;
    if is_key_down(KeyCode::S) {
        pos.translation.y += CHEAT_MOVE_SPEED;
        used = true;
    }
    if is_key_down(KeyCode::W) {
        pos.translation.y -= CHEAT_MOVE_SPEED;
        used = true;
    }
    if is_key_down(KeyCode::D) {
        pos.translation.x += CHEAT_MOVE_SPEED;
        used = true;
    }
    if is_key_down(KeyCode::A) {
        pos.translation.x -= CHEAT_MOVE_SPEED;
        used = true;
    }

    if is_key_pressed(KeyCode::RightBracket)
        && let Some(respawn_pos) =
            find_closest_respawn(world, pos.translation.into(), |respawn_pos| {
                respawn_pos.x > pos.translation.x + pixel_to_meter(50.0)
            })
    {
        pos.translation = respawn_pos.into();
        used = true;
    }
    if is_key_pressed(KeyCode::LeftBracket)
        && let Some(respawn_pos) =
            find_closest_respawn(world, pos.translation.into(), |respawn_pos| {
                respawn_pos.x < pos.translation.x - pixel_to_meter(50.0)
            })
    {
        pos.translation = respawn_pos.into();
        used = true;
    }
    let rigid_body = world.physics_world.get_body_mut(handle).unwrap();
    if used {
        rigid_body.set_position(pos, true);
        rigid_body.set_linvel(Vec2::ZERO.into(), true);
    }
}
async fn player_cheat_assets(assets: &mut Assets, world: &mut World) {
    if is_key_pressed(KeyCode::P) {
        println!("reloading assets...");
        *assets = Assets::new().await;
        for (level, pos) in world.levels.clone() {
            unload_level(world, level);
            load_level_at_pos(assets, world, level, pos);
        }
    }
}
fn find_closest_respawn(world: &World, pos: Vec2, filt: impl Fn(&Vec2) -> bool) -> Option<Vec2> {
    world
        .entities
        .query::<(&Respawn, &RigidBodyHandle)>()
        .iter()
        .map(|(_, (_, handle))| {
            world
                .physics_world
                .get_body(*handle)
                .unwrap()
                .position()
                .translation
        })
        .map(Vec2::from)
        .filter(filt)
        .min_by_key(|respawn_pos| OrderedFloat::from(respawn_pos.distance_squared(pos)))
}
fn player_movement(world: &mut World) {
    let alive = if let LifeState::Alive(Transition::End) = world.player.life_state {
        true
    } else {
        false
    };
    let polly = world.player.body.unwrap_polly_mut();
    let [
        left_feet_grounded,
        center_feet_grounded,
        right_feet_grounded,
    ] = polly.feet_grounded;

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
        // rotate towards 0
        angvel = 0.0;
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
        if let LifeState::Alive(Transition::End) = world.player.life_state
            && let Body::Polly(_) = world.player.body
        {
            world.player.life_state = LifeState::Dead(Transition::Start);
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

    let (pos, rotation) = find_respawn(world, world.player.respawn());
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
    load_level(assets, world, world.player.respawn().0);

    let (pos, _) = find_respawn(world, world.player.respawn());
    world.camera.target = pos;

    update_loaded_levels(assets, world);
}

fn find_respawn(world: &World, respawn: (LevelId, ThingId)) -> (Vec2, f32) {
    let (level_id, thing_id) = respawn;
    world
        .entities
        .query::<(&Respawn, &LevelId, &ThingId, &RigidBodyHandle)>()
        .into_iter()
        .filter(|&(_, (_, &level, _, _))| level == level_id)
        .filter(|&(_, (_, _, &thing, _))| thing == thing_id)
        .map(|(_, (_, _, _, pos))| {
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
fn get_player_body_mut(world: &mut World) -> &mut RigidBody {
    let body = world.player.body.any_body_handle();
    world.physics_world.get_body_mut(body).unwrap()
}

fn player_respawn(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    let body = get_player_body(world);
    let player_pos: Vec2 = (*body.translation()).into();
    for (_, (respawn, _, _, level_id, thing_id, _light_group)) in world
        .entities
        .query_mut::<(
            &mut Respawn,
            &RigidBodyHandle,
            &AreaOfEffect,
            &LevelId,
            &ThingId,
            &mut LightGroup,
        )>()
        .into_iter()
        .filter(|(_, (_, handle, area, _, _, _))| {
            area.contains(handle, &world.physics_world, player_pos)
        })
    {
        world.player.set_respawn((*level_id, *thing_id));

        respawn.active = match respawn.active.clone() {
            RespawnActive::Inactive => RespawnActive::Active(Transition::running(
                RESPAWN_ACTIVE_TRANSITION_DURATION,
                true,
            )),
            other => other,
        }
    }
    if is_key_pressed(KeyCode::R) {
        world.player.life_state = LifeState::Dead(Transition::Start);
    }
}
fn player_mushroom(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    let body = get_player_body(world);
    let mut linvel: Vec2 = (*body.linvel()).into();
    let player_pos: Vec2 = (*body.translation()).into();
    for (_, (mushroom, mushroom_handle)) in world
        .entities
        .query_mut::<(&mut Mushroom, &RigidBodyHandle)>()
    {
        let mushroom_body = world.physics_world.get_body(*mushroom_handle).unwrap();
        let mushroom_pos: Vec2 = (*mushroom_body.translation()).into();
        if player_pos.distance(mushroom_pos) < 0.2 {
            if mushroom.touching_player {
                continue;
            }
            mushroom.touching_player = true;
            let dir = Vec2::from_angle(mushroom.rotation).rotate(Vec2::new(0.0, -1.0));

            let current_speed_in_dir = linvel.dot(dir);

            let velocity_delta = PLAYER_VEL_MUSHROOM - current_speed_in_dir;

            linvel += dir * velocity_delta;
        } else {
            mushroom.touching_player = false;
        }
    }
    let body = get_player_body_mut(world);
    body.set_linvel(linvel.into(), true);
}
fn player_flytrap(world: &mut World) {
    if !world.player.alive() {
        return;
    }
    let body = get_player_body(world);
    let mut angvel: f32 = body.angvel();
    let player_pos: Vec2 = (*body.translation()).into();
    let player_is_rolly = world.player.body.is_rolly();
    for (_, (flytrap, flytrap_handle)) in world
        .entities
        .query_mut::<(&mut Flytrap, &RigidBodyHandle)>()
    {
        let flytrap_body = world.physics_world.get_body(*flytrap_handle).unwrap();
        let flytrap_pos: Vec2 = (*flytrap_body.translation()).into();
        if player_pos.distance(flytrap_pos) < 0.2 {
            if flytrap.touching_player {
                continue;
            }
            flytrap.touching_player = true;
            if player_is_rolly {
                angvel = if flytrap.flipped {
                    -PLAYER_VEL_FLYTRAP
                } else {
                    PLAYER_VEL_FLYTRAP
                };
                flytrap.teeth_speed = 2.0;
            } else {
                flytrap.teeth_speed = 1.0;
            }
        } else {
            flytrap.touching_player = false;
        }
    }
    let body = get_player_body_mut(world);
    body.set_angvel(angvel, true);
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
    for (entity, (lazy_collider, handle, thing_draw)) in world
        .entities
        .query::<(
            &mut LazyCollider,
            Option<&ColliderHandle>,
            Option<&ThingDraw>,
        )>()
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
        let texture = thing_draw.map(|x| x.texture.as_str()).unwrap_or("none");
        if let Some(handle) = handle {
            if !overlaps {
                if texture.contains("bamboo") {
                    println!("unloading, {:?}", texture);
                }
                world.physics_world.remove_collider(*handle);
                entities_remove_collider.push(entity);
            }
        } else {
            if overlaps {
                if texture.contains("bamboo") {
                    println!("loading, {:?}", texture);
                }
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

fn update_light(world: &mut World) {
    // let player_pos: Vec2 = (*body.translation()).into();
    let ripples: Vec<_> = world
        .entities
        .query::<&Ripple>()
        .iter()
        .map(|(_, r)| r)
        .cloned()
        .collect();
    for (_, (body, light_group)) in world
        .entities
        .query::<(&RigidBodyHandle, &mut LightGroup)>()
        .iter()
    {
        let body = world.physics_world.get_body(*body).unwrap();
        let pos = Vec2::from(body.position().translation.vector);
        let angle = body.position().rotation.angle();
        for (light, light_state) in light_group.lights.iter_mut() {
            let pos = pos + light.pos.rotate(Vec2::from_angle(angle));
            match light_state {
                LightState::Flicker(flicker) => flicker.update(get_frame_time()),
                LightState::Ripple(ripple) => ripple.update(get_frame_time(), pos, &ripples),
            }
        }
    }
}
fn update_back(world: &mut World, assets: &Assets) {
    world.back.update(get_frame_time());
    let player_pos = Vec2::from(get_player_body(world).position().translation);
    let current_level = world
        .levels
        .iter()
        .filter(|(level_id, level_pos)| {
            let dims = assets.levels[&level_id.0].0.dims;
            player_pos.x > level_pos.x
                && player_pos.x < level_pos.x + dims.x
                && player_pos.y > level_pos.y
                && player_pos.y < level_pos.y + dims.y
        })
        .map(|(level_id, _)| *level_id)
        .next();
    if let Some(level) = current_level {
        world.back.set_target(level);
    }
}
