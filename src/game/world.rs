use self::level::load_level;

use hecs::World as HecsWorld;
use macroquad::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
pub mod player;
use player::Player;

use super::texture::TextureHolder;
pub mod draw;
pub mod frame;
pub mod level;
pub mod polygon;
pub mod update;

pub struct World {
    pub player: Player,
    pub entities: HecsWorld,
    pub camera_target: Vec2,
    pub physics_world: PhysicsWorld,
}

impl World {
    pub fn new(texture_holder: &TextureHolder) -> Self {
        println!("Loading world...");
        let mut physics_world = PhysicsWorld::new();
        let camera_target = vec2(0.0, 0.0);
        let entities = HecsWorld::new();

        let player = Player::spawn(&mut physics_world);

        let mut world = Self {
            player,
            entities,
            camera_target,
            physics_world,
        };

        load_level(texture_holder, &mut world, 0);

        world
    }
}
