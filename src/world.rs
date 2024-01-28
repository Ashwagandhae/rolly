use self::level::load_level;

use super::consts::*;
use hecs::World as HecsWorld;
use macroquad::prelude::*;
use rapier2d::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
pub mod player;
use player::Player;
pub mod texture;
use texture::TextureHolder;
pub mod frame;
pub mod level;
pub struct World {
    pub player: Player,
    pub entities: HecsWorld,
    pub camera: Camera2D,
    pub zoom: f32,
    pub physics_world: PhysicsWorld,
    pub texture_holder: TextureHolder,
}

impl World {
    pub async fn new() -> Self {
        println!("Loading world...");
        let mut physics_world = PhysicsWorld::new();
        let camera = Camera2D {
            zoom: vec2(1. * ZOOM, screen_width() / screen_height() * ZOOM),
            target: vec2(0.0, 0.0),
            ..Default::default()
        };
        let entities = HecsWorld::new();

        let player = Player::spawn(&mut physics_world);

        let texture_holder = TextureHolder::new().await;

        let zoom = 1.0;

        let mut world = Self {
            player,
            entities,
            camera,
            physics_world,
            texture_holder,
            zoom,
        };

        load_level(0, &mut world);

        world
    }
}
