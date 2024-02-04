use std::collections::HashMap;

use crate::game::world::{frame::Transition, level::LevelId, update::camera_zoom};

use self::level::load_level;

use hecs::World as HecsWorld;
use macroquad::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
pub mod player;
use life_state::LifeState;
use player::Player;

use super::{assets::Assets, ui::settings::Settings};
pub mod collider;
pub mod draw;
pub mod frame;
pub mod level;
pub mod life_state;
pub mod polygon;
pub mod svg;
pub mod thing;
pub mod update;

pub struct World {
    pub player: Player,
    pub entities: HecsWorld,
    pub camera: Camera2D,
    pub physics_world: PhysicsWorld,
    pub levels: HashMap<LevelId, Vec2>,
}

impl World {
    pub fn new(settings: &Settings, assets: &Assets) -> Self {
        println!("Loading world...");
        let mut physics_world = PhysicsWorld::new();
        let camera = Camera2D {
            target: vec2(0.0, 3.0),
            ..Default::default()
        };
        let entities = HecsWorld::new();

        let player = Player::spawn(&mut physics_world);

        let levels = HashMap::new();

        let mut world = Self {
            player,
            entities,
            camera,
            physics_world,
            levels,
        };

        load_level(assets, &mut world, LevelId(0));

        update::update(assets, settings, &mut world);

        world
    }
}
