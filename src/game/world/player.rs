use super::frame::ContinuousFrame;
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
        let (body_handle, collider_handle) = physics_world.add_body(
            RigidBodyBuilder::dynamic()
                .translation(translation.into())
                .rotation(rotation)
                .linvel(linvel.into())
                .angvel(angvel)
                .linear_damping(PLAYER_LINEAR_DAMPING)
                .angular_damping(PLAYER_ANGULAR_DAMPING)
                .ccd_enabled(true)
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
        let (body_handle, collider_handle) = physics_world.add_body(
            RigidBodyBuilder::dynamic()
                .translation(translation.into())
                .rotation(rotation)
                .linvel(linvel.into())
                .angvel(angvel)
                .linear_damping(PLAYER_ROLLY_LINEAR_DAMPING)
                .angular_damping(PLAYER_ROLLY_ANGULAR_DAMPING)
                .ccd_enabled(true)
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
}

#[derive(Debug, Clone)]
pub enum Transition {
    Start,
    End,
    Between { time: f32, duration: f32, rev: bool },
}

impl Transition {
    pub fn tick(&mut self, delta_time: f32) {
        *self = match self.clone() {
            Self::Start => Self::Start,
            Self::End => Self::End,
            Self::Between {
                mut time,
                duration,
                rev,
            } => {
                time += delta_time / duration * if rev { -1.0 } else { 1.0 };
                if time < 0.0 {
                    Self::End
                } else if time > 1.0 {
                    Self::Start
                } else {
                    Self::Between {
                        time,
                        duration,
                        rev,
                    }
                }
            }
        }
    }
    pub fn run(&mut self, duration: f32, rev: bool) {
        *self = match (self.clone(), rev) {
            (no_change @ Self::Start, false) | (no_change @ Self::End, true) => no_change,
            (Self::Start, true) | (Self::End, false) => Self::Between {
                time: if rev { 1.0 } else { 0.0 },
                duration,
                rev,
            },
            (Self::Between { time, .. }, _) => Self::Between {
                time,
                duration,
                rev,
            },
        }
    }
}

pub struct Tween {
    pub target: f32,
    pub value: f32,
    pub half_life: f32,
}

impl Tween {
    pub fn new(value: f32, half_life: f32) -> Self {
        Self {
            target: value,
            value,
            half_life,
        }
    }
    pub fn tick(&mut self, delta_time: f32) {
        self.value += (self.target - self.value) * (delta_time / self.half_life) * 0.5;
    }
    pub fn set(&mut self, value: f32) {
        self.target = value;
    }
    pub fn get(&self) -> f32 {
        self.value
    }
}

pub struct Player {
    pub direction: Direction,
    pub body: Body,
    pub rolly_polly_transition: Transition,
    pub eye_x: Tween,
}
impl Player {
    pub fn spawn(physics_world: &mut PhysicsWorld) -> Self {
        let direction = Direction::Right;

        let body = Body::Polly(Polly::spawn(
            physics_world,
            vec2(0.0, 0.0),
            0.0,
            vec2(0.0, 0.0),
            0.0,
        ));

        let rolly_polly_transition = Transition::End;

        let eye_x = Tween::new(1.0, 0.05);

        Self {
            direction,
            body,
            rolly_polly_transition,
            eye_x,
        }
    }
}
