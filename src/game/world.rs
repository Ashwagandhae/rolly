use std::collections::HashMap;

use crate::game::{
    config::GameConfig,
    world::{back::Back, level::LevelId},
};

use self::level::load_level;

use hecs::World as HecsWorld;
use macroquad::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
pub mod player;
use player::Player;

use super::{assets::Assets, ui::settings::Settings};
pub mod back;
pub mod collider;
pub mod draw;
pub mod frame;
pub mod level;
pub mod life_state;
pub mod light;
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
    pub back: Back,
}

impl World {
    pub fn new(_settings: &Settings, assets: &mut Assets, config: &GameConfig) -> Self {
        println!("Loading world...");
        let mut physics_world = PhysicsWorld::new();
        let camera = Camera2D {
            target: vec2(0.0, 3.0),
            ..Default::default()
        };
        let start_level = LevelId(config.level.unwrap_or(0));
        let entities = HecsWorld::new();

        let player = Player::spawn(&mut physics_world, start_level);

        let levels = HashMap::new();

        let back = Back::new(start_level);

        let mut world = Self {
            player,
            entities,
            camera,
            physics_world,
            levels,
            back,
        };

        load_level(assets, &mut world, start_level);

        world
    }
}
