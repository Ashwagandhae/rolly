use std::ops::Mul;

use hecs::{BuiltEntity, Component, EntityBuilder as HecsEntityBuilder};
use macroquad::prelude::*;
use rapier2d::prelude::*;

use super::{
    draw::meter_to_pixel,
    floor::{LazyCollider, Material},
    level::LevelId,
    light::{load_light, Lights},
    physics_world::PhysicsWorld,
    World,
};
use crate::{
    consts::*,
    game::{assets::Assets, world::collider},
};

#[derive(Debug, Clone)]
pub enum ThingName {
    Stone,
    Spike,
    RespawnGrass,
    RespawnMud,
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
        0x938260 => RespawnMud,
        _ => return None,
    })
}

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
    let tx = |world: &mut World, texture: &str, material: Material, ex: BasicThingParams| {
        basic_thing_ex(assets, world, pos, rotation, texture, material, ex)
    };
    let _m = |pos: Vec2| basic_marker(pos);
    match name {
        ThingName::Stone => t(world, "stone", Material::Stone),
        ThingName::Spike => t(world, "spike", Material::Stone),
        ThingName::RespawnGrass => tx(
            world,
            "respawn-grass",
            Material::Grass,
            BasicThingParams {
                light: LightRepr::DefaultFile,
                ..Default::default()
            },
        )
        .add(Respawn {})
        .add(AreaOfEffect::new(RESPAWN_AQUIRE_RADIUS)),
        ThingName::RespawnMud => tx(
            world,
            "respawn-mud",
            Material::Mud,
            BasicThingParams {
                light: LightRepr::DefaultFile,
                ..Default::default()
            },
        )
        .add(Respawn {})
        .add(AreaOfEffect::new(RESPAWN_AQUIRE_RADIUS)),
    }
}

pub struct Respawn {}

pub struct AreaOfEffect {
    pub radius: f32,
}

impl AreaOfEffect {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
    pub fn contains(
        &self,
        body: &RigidBodyHandle,
        physics_world: &PhysicsWorld,
        target_pos: Vec2,
    ) -> bool {
        let body = physics_world.get_body(*body).unwrap();
        let pos = body.position().translation.vector;
        let distance = (Vec2::from(pos) - target_pos).length();
        distance < self.radius
    }
}

#[derive(Clone)]
pub enum ColliderRepr {
    None,
    DefaultFile,
    File(String),
    Raw(Rect, ColliderBuilder),
}

#[derive(Debug, Clone)]
pub enum LightRepr {
    None,
    DefaultFile,
    File(String),
    Raw(Lights),
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
    pub collider: ColliderRepr,
    pub light: LightRepr,
    pub lazy: bool,
}

impl std::default::Default for BasicThingParams {
    fn default() -> Self {
        Self {
            collider: ColliderRepr::DefaultFile,
            light: LightRepr::None,
            lazy: true,
        }
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
    let body = RigidBodyBuilder::fixed()
        .translation(pos.into())
        .rotation(rotation);

    let body_handle = world.physics_world.add_body(body.build());

    let mut builder = EntityBuilder::new()
        .add(body_handle)
        .add(ThingDraw {
            texture: texture.to_owned(),
        })
        .add(material);

    let collider = match ex.collider {
        ColliderRepr::DefaultFile => Some(collider::load_collider(assets, &texture)),
        ColliderRepr::File(collider_file) => Some(collider::load_collider(assets, &collider_file)),
        ColliderRepr::Raw(rect, builder) => Some((rect, builder)),
        ColliderRepr::None => None,
    };

    if let Some((rect, collider)) = collider {
        let collider = collider
            .friction(PLATFORM_FRICTION)
            .friction_combine_rule(CoefficientCombineRule::Max)
            .sensor(!material.rigid());

        if ex.lazy {
            let rect = Rect::new(pos.x - rect.w / 2.0, pos.y - rect.h / 2.0, rect.w, rect.h);
            builder = builder.add(LazyCollider {
                rect,
                builder: collider,
                body_handle,
            })
        } else {
            let handle = world
                .physics_world
                .add_collider(collider.build(), body_handle);
            builder = builder.add(handle);
        }
    }

    let light = match ex.light {
        LightRepr::DefaultFile => Some(load_light(assets, &texture)),
        LightRepr::File(light_file) => Some(load_light(assets, &light_file)),
        LightRepr::Raw(light) => Some(light),
        LightRepr::None => None,
    };
    if let Some(light) = light {
        builder = builder.add(light);
    }
    builder
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
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ThingId(pub usize);

pub fn spawn_thing(
    assets: &Assets,
    world: &mut World,
    thing_info: ThingInfo,
    level: LevelId,
    thing_id: ThingId,
    pos: Vec2,
) {
    let Some(name) = thing_info_to_name(thing_info.clone()) else {
        return;
    };
    let mut entity =
        thing_name_to_entity(assets, world, name, thing_info.pos + pos, thing_info.rotate)
            .add(level)
            .add(thing_id);
    world.entities.spawn(entity.build());
}
