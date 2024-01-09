use macroquad::prelude::*;
use rapier2d::prelude::*;

pub struct Floor {
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub vertices: Vec<Vec2>,
}
