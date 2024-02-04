use super::frame::{ContinuousFrame, Transition, Tween};
use super::level::LevelId;
use super::life_state::LifeState;
use super::physics_world::PhysicsWorld;
use crate::consts::*;
use macroquad::prelude::*;
use rapier2d::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Polly {
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub feet_sensor_handles: [ColliderHandle; 3],
    pub feet_grounded: [bool; 3],
    pub feet_frame: ContinuousFrame,
}

impl Polly {
    pub fn spawn(
        physics_world: &mut PhysicsWorld,
        translation: Vec2,
        rotation: f32,
        linvel: Vec2,
        angvel: f32,
    ) -> Self {
        let (body_handle, collider_handle) = physics_world.add_body_and_collider(
            RigidBodyBuilder::dynamic()
                .translation(translation.into())
                .rotation(rotation)
                .linvel(linvel.into())
                .angvel(angvel)
                .linear_damping(PLAYER_LINEAR_DAMPING)
                .angular_damping(PLAYER_ANGULAR_DAMPING)
                .ccd_enabled(CCD_ENABLED)
                .can_sleep(false)
                .build(),
            ColliderBuilder::capsule_x(0.05, 0.05)
                .friction(PLAYER_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Max)
                .build(),
        );

        let mut make_foot = |builder: ColliderBuilder, offset: Vec2| {
            physics_world.collider_set.insert_with_parent(
                builder.position(offset.into()).sensor(true).build(),
                body_handle,
                &mut physics_world.rigid_body_set,
            )
        };

        let feet_center = make_foot(ColliderBuilder::cuboid(0.1, 0.01), vec2(0.0, 0.05));
        let feet_left = make_foot(ColliderBuilder::ball(0.01), vec2(-0.1, 0.05));
        let feet_right = make_foot(ColliderBuilder::ball(0.01), vec2(0.1, 0.05));

        let feet_sensor_handles = [feet_left, feet_center, feet_right];
        let feet_grounded = [false; 3];

        let feet_frame = ContinuousFrame::new(0.0);

        Self {
            collider_handle,
            body_handle,
            feet_sensor_handles,
            feet_grounded,
            feet_frame,
        }
    }
    pub fn despawn(&self, physics_world: &mut PhysicsWorld) {
        physics_world.remove_body(self.body_handle);
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Rolly {
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

impl Rolly {
    pub fn spawn(
        physics_world: &mut PhysicsWorld,
        translation: Vec2,
        rotation: f32,
        linvel: Vec2,
        angvel: f32,
    ) -> Self {
        let (body_handle, collider_handle) = physics_world.add_body_and_collider(
            RigidBodyBuilder::dynamic()
                .translation(translation.into())
                .rotation(rotation)
                .linvel(linvel.into())
                .angvel(angvel)
                .linear_damping(PLAYER_ROLLY_LINEAR_DAMPING)
                .angular_damping(PLAYER_ROLLY_ANGULAR_DAMPING)
                .ccd_enabled(CCD_ENABLED)
                .can_sleep(false)
                .build(),
            ColliderBuilder::ball(0.075)
                .friction(PLAYER_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Max)
                .build(),
        );
        Rolly {
            body_handle,
            collider_handle,
        }
    }
    pub fn despawn(&self, physics_world: &mut PhysicsWorld) {
        physics_world.remove_body(self.body_handle);
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Polly(Polly),
    Rolly(Rolly),
}

impl Body {
    pub fn any_body_handle(&self) -> RigidBodyHandle {
        match self {
            Body::Polly(polly) => polly.body_handle,
            Body::Rolly(rolly) => rolly.body_handle,
        }
    }
    pub fn any_collider_handle(&self) -> ColliderHandle {
        match self {
            Body::Polly(polly) => polly.collider_handle,
            Body::Rolly(rolly) => rolly.collider_handle,
        }
    }
    pub fn unwrap_polly(&self) -> &Polly {
        match self {
            Body::Polly(polly) => polly,
            Body::Rolly(_) => panic!("unwrap_poll called on Body::Rolly"),
        }
    }
    pub fn unwrap_polly_mut(&mut self) -> &mut Polly {
        match self {
            Body::Polly(polly) => polly,
            Body::Rolly(_) => panic!("unwrap_poll called on Body::Rolly"),
        }
    }

    pub fn unwrap_rolly(&self) -> &Rolly {
        match self {
            Body::Rolly(rolly) => rolly,
            Body::Polly(_) => panic!("unwrap_rolly called on Body::Polly"),
        }
    }

    pub fn unwrap_rolly_mut(&mut self) -> &mut Rolly {
        match self {
            Body::Rolly(rolly) => rolly,
            Body::Polly(_) => panic!("unwrap_rolly called on Body::Polly"),
        }
    }

    pub fn despawn(&self, physics_world: &mut PhysicsWorld) {
        match self {
            Body::Polly(polly) => polly.despawn(physics_world),
            Body::Rolly(rolly) => rolly.despawn(physics_world),
        }
    }
}

pub struct Player {
    pub direction: Direction,
    pub body: Body,
    pub rolly_polly_transition: Transition,
    pub eye_x: Tween,
    pub life_state: LifeState,
    pub respawn: LevelId,
}
impl Player {
    pub fn spawn(physics_world: &mut PhysicsWorld) -> Self {
        let body = Body::Rolly(Rolly::spawn(
            physics_world,
            vec2(0.0, 0.0),
            0.0,
            vec2(0.0, 0.0),
            0.0,
        ));

        let direction = Direction::Right;
        let rolly_polly_transition = Transition::End;
        let eye_x = Tween::new(1.0, 0.05);

        let life_state = LifeState::Dead(Transition::End);
        let respawn = LevelId::first();

        Self {
            direction,
            body,
            rolly_polly_transition,
            eye_x,
            life_state,
            respawn,
        }
    }
    pub fn alive(&self) -> bool {
        matches!(self.life_state, LifeState::Alive(Transition::End))
    }
    /// reset everything except life_state, respawn, and any physics_world state
    pub fn reset(&mut self, physics_world: &mut PhysicsWorld) {
        self.body.despawn(physics_world);

        self.body = Body::Rolly(Rolly::spawn(
            physics_world,
            vec2(0.0, 0.0),
            0.0,
            vec2(0.0, 0.0),
            0.0,
        ));

        self.direction = Direction::Right;
        self.rolly_polly_transition = Transition::End;
        self.eye_x = Tween::new(1.0, 0.05);
    }
}
