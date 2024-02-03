use std::ops::Mul;

use hecs::{BuiltEntity, Component, EntityBuilder as HecsEntityBuilder};
use macroquad::prelude::*;
use rapier2d::prelude::*;

use super::{draw::meter_to_pixel, floor::Material, level::LevelId, World};
use crate::{
    consts::*,
    game::{assets::Assets, world::collider},
};

#[derive(Debug, Clone)]
pub enum ThingName {
    Stone,
    Spike,
    RespawnGrass,
}
pub fn thing_info_to_name(info: ThingInfo) -> Option<ThingName> {
    use ThingName::*;
    use UsizeShapeSize::*;
    let size = UsizeShapeSize::from_thing_info(&meter_to_pixel(info.size));
    let color = info.color;
    Some(match color {
        0x495380 => match size {
            Rect(200, _) => Stone,
            _ => Spike,
        },
        0xCCCFAA => RespawnGrass,
        _ => return None,
    })
}
pub struct Respawn {}

pub fn thing_name_to_entity(
    assets: &Assets,
    world: &mut World,
    name: ThingName,
    pos: Vec2,
    rotation: f32,
) -> EntityBuilder {
    let t = |world: &mut World, texture: &str, material: Material| {
        basic_thing(assets, world, pos, rotation, texture, material)
    };
    let _m = |pos: Vec2| basic_marker(pos);
    match name {
        ThingName::Stone => t(world, "stone", Material::Stone),
        ThingName::Spike => t(world, "spike", Material::Stone),
        ThingName::RespawnGrass => t(world, "respawn-grass", Material::Grass).add(Respawn {}),
    }
}

#[derive(Clone)]
pub enum ColliderRepr {
    File(String),
    Raw(ColliderBuilder),
}

#[derive(Debug, Clone)]
enum UsizeShapeSize {
    Rect(usize, usize),
    Circle(usize),
}

impl UsizeShapeSize {
    pub fn from_thing_info(thing_info_shape_size: &ThingInfoShapeSize) -> Self {
        match thing_info_shape_size {
            ThingInfoShapeSize::Rect(dims) => {
                UsizeShapeSize::Rect(dims.x.round() as usize, dims.y.round() as usize)
            }
            ThingInfoShapeSize::Circle(radius) => UsizeShapeSize::Circle(radius.round() as usize),
        }
    }
}

#[derive(Clone)]
pub struct BasicThingParams {
    pub collider: Option<ColliderRepr>,
}

impl std::default::Default for BasicThingParams {
    fn default() -> Self {
        Self { collider: None }
    }
}

pub struct EntityBuilder(HecsEntityBuilder);
impl EntityBuilder {
    pub fn new() -> Self {
        Self(HecsEntityBuilder::new())
    }

    pub fn add<T: Component>(mut self, component: T) -> Self {
        self.0.add(component);
        self
    }

    pub fn build(&mut self) -> BuiltEntity<'_> {
        self.0.build()
    }
}
fn basic_thing_ex(
    assets: &Assets,
    world: &mut World,
    pos: Vec2,
    rotation: f32,
    texture: &str,
    material: Material,
    ex: BasicThingParams,
) -> EntityBuilder {
    let collider = match ex
        .collider
        .unwrap_or(ColliderRepr::File(texture.to_owned()))
    {
        ColliderRepr::File(collider_file) => collider::load_collider(assets, &collider_file),
        ColliderRepr::Raw(builder) => builder,
    }
    .friction(PLATFORM_FRICTION)
    .friction_combine_rule(CoefficientCombineRule::Max)
    .sensor(!material.rigid());

    let body = RigidBodyBuilder::fixed()
        .translation(pos.into())
        .rotation(rotation);

    let (body_handle, collider_handle) =
        world.physics_world.add_body(body.build(), collider.build());

    EntityBuilder::new()
        .add(body_handle)
        .add(collider_handle)
        .add(ThingDraw {
            texture: texture.to_owned(),
        })
        .add(material)
}
fn basic_thing(
    assets: &Assets,
    world: &mut World,
    pos: Vec2,
    rotation: f32,
    texture: &str,
    material: Material,
) -> EntityBuilder {
    basic_thing_ex(
        assets,
        world,
        pos,
        rotation,
        texture,
        material,
        BasicThingParams::default(),
    )
}

fn basic_marker(pos: Vec2) -> EntityBuilder {
    EntityBuilder::new().add(pos)
}

#[derive(Debug, Clone)]
pub struct ThingDraw {
    pub texture: String,
}

#[derive(Debug, Clone)]
pub struct ThingInfo {
    pos: Vec2,
    rotate: f32,
    size: ThingInfoShapeSize,
    color: u32,
}

#[derive(Debug, Clone)]
pub enum ThingInfoShapeSize {
    Rect(Vec2),
    Circle(f32),
}

impl Mul<f32> for ThingInfoShapeSize {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        match self {
            ThingInfoShapeSize::Rect(dims) => ThingInfoShapeSize::Rect(dims * rhs),
            ThingInfoShapeSize::Circle(radius) => ThingInfoShapeSize::Circle(radius * rhs),
        }
    }
}

impl ThingInfo {
    pub fn new_rect(pos: Vec2, rotate: f32, dims: Vec2, color: u32) -> Self {
        Self {
            pos,
            rotate,
            size: ThingInfoShapeSize::Rect(dims),
            color,
        }
    }

    pub fn new_circle(pos: Vec2, rotate: f32, radius: f32, color: u32) -> Self {
        Self {
            pos,
            rotate,
            size: ThingInfoShapeSize::Circle(radius),
            color,
        }
    }
}
pub fn spawn_thing(
    assets: &Assets,
    world: &mut World,
    thing_info: ThingInfo,
    level: LevelId,
    pos: Vec2,
) {
    let Some(name) = thing_info_to_name(thing_info.clone()) else {
        return;
    };
    let mut entity =
        thing_name_to_entity(assets, world, name, thing_info.pos + pos, thing_info.rotate)
            .add(level);
    world.entities.spawn(entity.build());
}
