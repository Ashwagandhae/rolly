use std::collections::HashMap;

use crate::game::world::level::LevelId;

use self::level::{load_level, LevelInfo};

use hecs::World as HecsWorld;
use macroquad::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
pub mod player;
use player::Player;

use super::assets::Assets;
pub mod collider;
pub mod draw;
pub mod frame;
pub mod level;
pub mod polygon;
pub mod svg;
pub mod thing;
pub mod update;

pub struct World {
    pub player: Player,
    pub entities: HecsWorld,
    pub camera_target: Vec2,
    pub physics_world: PhysicsWorld,
    pub levels: HashMap<LevelId, Vec2>,
}

impl World {
    pub fn new(assets: &Assets) -> Self {
        println!("Loading world...");
        let mut physics_world = PhysicsWorld::new();
        let camera_target = vec2(0.0, 0.0);
        let entities = HecsWorld::new();

        let player = Player::spawn(&mut physics_world);

        let levels = HashMap::new();

        let mut world = Self {
            player,
            entities,
            camera_target,
            physics_world,
            levels,
        };

        load_level(assets, &mut world, LevelId(0), vec2(0.0, 0.0));

        world
    }
}
