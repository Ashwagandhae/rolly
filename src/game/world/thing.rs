use std::ops::Mul;

use hecs::{BuiltEntity, Component, EntityBuilder as HecsEntityBuilder};
use macroquad::prelude::*;
use rapier2d::prelude::*;

use super::{
    World,
    floor::{LazyCollider, Material},
    level::LevelId,
    light::{FlickerState, LightGroup, LightState, load_light},
    physics_world::PhysicsWorld,
};
use crate::{
    consts::*,
    game::{
        assets::Assets,
        world::{
            collider,
            draw::pixel_to_meter,
            frame::Transition,
            level::DrawLayer,
            light::{RippleSource, RippleState},
        },
    },
};

pub struct Mushroom {
    pub touching_player: bool,
    pub rotation: f32,
}

pub fn thing_info_to_entity(
    assets: &Assets,
    world: &mut World,
    shape_size: ThingInfoShapeSize,
    color: u32,
    pos: Vec2,
    rotation: f32,
    level_id: LevelId,
    thing_id: ThingId,
) -> Option<Vec<EntityBuilder>> {
    use ThingInfoShapeSize::*;
    let t = |world: &mut World, texture: &str, material: Material| {
        basic_thing(
            assets,
            world,
            pos,
            rotation,
            texture,
            material,
            BasicThingParams::default(),
        )
    };
    let tx = |world: &mut World, texture: &str, material: Material, ex: BasicThingParams| {
        basic_thing(assets, world, pos, rotation, texture, material, ex)
    };
    Some(match color {
        0x495380 => match shape_size {
            Rect(size) if size.x >= pixel_to_meter(150.0) => {
                vec![t(world, "stone", Material::Stone)]
            }
            _ => vec![t(world, "spike", Material::Stone)],
        },

        0x409F84 => vec![tx(
            world,
            "fern",
            Material::Fern,
            BasicThingParams {
                collider: ColliderRepr::None,
                ..Default::default()
            },
        )],
        0xCCCFAA => vec![respawn_thing(
            assets,
            world,
            pos,
            rotation,
            "respawn-grass",
            Material::Grass,
            level_id,
            thing_id,
        )],
        0x938260 => vec![respawn_thing(
            assets,
            world,
            pos,
            rotation,
            "respawn-mud",
            Material::Mud,
            level_id,
            thing_id,
        )],
        0xAAC4CF => vec![respawn_thing(
            assets,
            world,
            pos,
            rotation,
            "respawn-stone",
            Material::Stone,
            level_id,
            thing_id,
        )],
        0xAACFB5 => vec![respawn_thing(
            assets,
            world,
            pos,
            rotation,
            "respawn-fern",
            Material::Fern,
            level_id,
            thing_id,
        )],
        0x93607A => vec![
            tx(
                world,
                "mushroom",
                Material::Mud,
                BasicThingParams {
                    collider: ColliderRepr::DefaultFile,
                    ..Default::default()
                },
            )
            .add(Mushroom {
                touching_player: false,
                rotation,
            }),
        ],
        0x1C7D46 => create_bamboo(assets, world, pos, rotation, shape_size.height()),
        _ => return None,
    })
}
fn create_bamboo_segment(
    assets: &Assets,
    world: &mut World,
    pos: Vec2,
    rotation: f32,
    height: f32,
    i: usize,
    num_segments: usize,
    prev_body_handle: Option<RigidBodyHandle>,
) -> (EntityBuilder, RigidBodyHandle) {
    let dir = vec2(0.0, -1.0).rotate(Vec2::from_angle(rotation));
    let pos = pos - dir * height / 2.0 + dir * pixel_to_meter(BAMBOO_SEGMENT_HEIGHT) / 2.0;

    let body = if prev_body_handle.is_some() {
        RigidBodyBuilder::dynamic()
    } else {
        RigidBodyBuilder::fixed()
    };
    let body = body
        .translation((pos + dir * pixel_to_meter(BAMBOO_SEGMENT_HEIGHT) * i as f32).into())
        .rotation(rotation);
    let body_handle = world.physics_world.add_body(body.build());
    let big_leaf_cutoff = num_segments * 3 / 5;
    let texture = if i < big_leaf_cutoff {
        format!("bamboo{}", rand::gen_range(0, 1))
    } else if i == big_leaf_cutoff {
        format!("bamboo-leaf-small{}", rand::gen_range(0, 1))
    } else {
        format!("bamboo-leaf-big{}", rand::gen_range(0, 1))
    };
    let mut builder = EntityBuilder::new()
        .add(body_handle)
        .add(ThingDraw {
            texture,
            ..Default::default()
        })
        .add(Material::Grass);

    let (_, collider) = collider::load_collider(&assets, "bamboo");

    let collider = environment_collider(collider, false);

    let handle = world
        .physics_world
        .add_collider(collider.build(), body_handle);
    builder = builder.add(handle);

    if let Some(prev_body_handle) = prev_body_handle {
        let anchor_fixed = point![0.0, pixel_to_meter(-110.0)];
        let anchor_dynamic = point![0.0, pixel_to_meter(90.0)];
        let joint_data = RevoluteJointBuilder::new()
            .local_anchor1(anchor_fixed)
            .local_anchor2(anchor_dynamic)
            .motor_position(0.0, 3000.0 * (num_segments - i + 1) as f32, 500.0);
        world.physics_world.impulse_joint_set.insert(
            prev_body_handle,
            body_handle,
            joint_data,
            true,
        );
    }
    (builder, body_handle)
}
fn create_bamboo(
    assets: &Assets,
    world: &mut World,
    pos: Vec2,
    rotation: f32,
    height: f32,
) -> Vec<EntityBuilder> {
    let num_segments = (height / pixel_to_meter(BAMBOO_SEGMENT_HEIGHT)).floor() as usize;
    let mut prev_body_handle = None;
    let mut res = Vec::new();
    let seed = pos.x;
    rand::srand(seed.to_bits() as u64);
    for i in 0..num_segments.max(1) {
        let (builder, handle) = create_bamboo_segment(
            assets,
            world,
            pos,
            rotation,
            height,
            i,
            num_segments,
            prev_body_handle,
        );
        prev_body_handle = Some(handle);
        res.push(builder)
    }
    res
}
fn respawn_thing(
    assets: &Assets,
    world: &mut World,
    target_pos: Vec2,
    rotation: f32,
    texture: &str,
    material: Material,
    level_id: LevelId,
    thing_id: ThingId,
) -> EntityBuilder {
    let down_dir = Vec2::from_angle(rotation).rotate(vec2(0.0, 1.0));
    let offset = down_dir * RESPAWN_INACTIVE_OFFSET;
    let starts_active = world.player.all_respawns().contains(&(level_id, thing_id));
    let light = load_light(
        assets,
        &texture,
        if starts_active {
            LightState::Ripple(RippleState { strength: 1.0 })
        } else {
            LightState::Flicker(FlickerState::Off)
        },
    );
    basic_thing(
        assets,
        world,
        target_pos,
        rotation,
        texture,
        material,
        BasicThingParams {
            // light: LightRepr::DefaultFile,
            ..Default::default()
        },
    )
    .add(light)
    .add(ThingDraw {
        texture: texture.to_owned(),
        offset: if starts_active { Vec2::ZERO } else { offset },
    })
    .add(Respawn {
        active: if starts_active {
            RespawnActive::Active(Transition::End)
        } else {
            RespawnActive::Inactive
        },
        target_pos,
        offset,
    })
    .add(RippleSource {
        just_contacted: false,
    })
    .add(AreaOfEffect::new(RESPAWN_AQUIRE_RADIUS))
}

#[derive(Debug, Clone)]
pub struct Respawn {
    pub active: RespawnActive,
    pub target_pos: Vec2,
    pub offset: Vec2,
}

#[derive(Debug, Clone)]
pub enum RespawnActive {
    Inactive,
    Active(Transition),
}

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
    Raw(LightGroup),
}

#[derive(Clone)]
pub struct BasicThingParams {
    pub collider: ColliderRepr,
    pub lazy: bool,
}

impl std::default::Default for BasicThingParams {
    fn default() -> Self {
        Self {
            collider: ColliderRepr::DefaultFile,
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
fn basic_thing(
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
            ..Default::default()
        })
        .add(material);

    let collider = match ex.collider {
        ColliderRepr::DefaultFile => Some(collider::load_collider(assets, &texture)),
        ColliderRepr::File(collider_file) => Some(collider::load_collider(assets, &collider_file)),
        ColliderRepr::Raw(rect, builder) => Some((rect, builder)),
        ColliderRepr::None => None,
    };

    if let Some((rect, collider)) = collider {
        let collider = environment_collider(collider, !material.rigid());

        if ex.lazy {
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

    builder
}
fn environment_collider(collider: ColliderBuilder, sensor: bool) -> ColliderBuilder {
    collider
        .friction(PLATFORM_FRICTION)
        .friction_combine_rule(CoefficientCombineRule::Max)
        .collision_groups(InteractionGroups::new(
            COLLISION_LAYER_ENVIRONMENT.into(),
            COLLISION_LAYER_PLAYER.into(),
        ))
        .sensor(sensor)
}

#[derive(Debug, Clone)]
pub struct ThingDraw {
    pub texture: String,
    pub offset: Vec2,
}
impl Default for ThingDraw {
    fn default() -> Self {
        ThingDraw {
            texture: "stone".to_owned(),
            offset: Vec2::ZERO,
        }
    }
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
impl ThingInfoShapeSize {
    pub fn height(&self) -> f32 {
        match self {
            Self::Rect(vec) => vec.y,
            Self::Circle(r) => r * 2.,
        }
    }
    pub fn width(&self) -> f32 {
        match self {
            Self::Rect(vec) => vec.x,
            Self::Circle(r) => r * 2.,
        }
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct ThingId(pub usize);

pub fn spawn_thing(
    assets: &Assets,
    world: &mut World,
    thing_info: ThingInfo,
    level: LevelId,
    thing_id: ThingId,
    pos: Vec2,
    draw_layer: DrawLayer,
) {
    let Some(entities) = thing_info_to_entity(
        assets,
        world,
        thing_info.size,
        thing_info.color,
        thing_info.pos + pos,
        thing_info.rotate,
        level,
        thing_id,
    ) else {
        return;
    };
    for entity in entities {
        let mut entity = entity.add(level).add(thing_id).add(draw_layer);
        world.entities.spawn(entity.build());
    }
}
