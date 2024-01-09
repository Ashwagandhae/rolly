use super::consts::*;
use macroquad::prelude::*;
use rapier2d::prelude::*;

pub mod physics_world;
use physics_world::PhysicsWorld;
pub mod floor;
use floor::Floor;
pub mod player;
use player::Player;
pub mod texture;
use texture::TextureHolder;
pub mod frame;
pub struct World {
    pub player: Player,
    pub floors: Vec<Floor>,
    pub camera: Camera2D,
    pub zoom: f32,
    pub physics_world: PhysicsWorld,
    pub texture_holder: TextureHolder,
}

impl World {
    pub async fn new() -> Self {
        let mut physics_world = PhysicsWorld::new();
        let camera = Camera2D {
            zoom: vec2(1. * ZOOM, screen_width() / screen_height() * ZOOM),
            target: vec2(0.0, 0.0),
            ..Default::default()
        };

        let floors = create_floors(
            &mut physics_world,
            vec![
                (
                    vec2(-0.2, 0.4),
                    vec![vec2(0.0, 0.0), vec2(0.0, -0.2), vec2(0.4, 0.0)],
                ),
                (
                    vec2(0.0, 0.8),
                    vec![vec2(-3.0, 0.0), vec2(-2.0, 0.0), vec2(-2.5, -0.1)],
                ),
                (
                    vec2(0.0, 0.8),
                    vec![
                        vec2(-3.0, 0.0),
                        vec2(-3.0, 0.1),
                        vec2(3.0, 0.1),
                        vec2(3.0, 0.0),
                    ],
                ),
            ],
        );

        let player = Player::spawn(&mut physics_world);

        let texture_holder = TextureHolder::new().await;

        let zoom = 1.0;

        Self {
            player,
            floors,
            camera,
            physics_world,
            texture_holder,
            zoom,
        }
    }
}

fn create_floors(
    physics_world: &mut PhysicsWorld,
    floor_vertices: Vec<(Vec2, Vec<Vec2>)>,
) -> Vec<Floor> {
    floor_vertices
        .into_iter()
        .map(|(start_pos, vertices)| {
            let builder = ColliderBuilder::convex_hull(
                &vertices.iter().map(|&v| v.into()).collect::<Vec<_>>(),
            )
            .unwrap()
            .friction(PLATFORM_FRICTION)
            .friction_combine_rule(CoefficientCombineRule::Max);
            let (body_handle, collider_handle) = physics_world.add_body(
                RigidBodyBuilder::fixed()
                    .translation(start_pos.into())
                    .build(),
                builder.build(),
            );
            Floor {
                body_handle,
                collider_handle,
                vertices,
            }
        })
        .collect()
}
