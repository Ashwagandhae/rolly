use macroquad::prelude::*;
use nalgebra::{Isometry, Isometry2, UnitComplex};
use rapier2d::prelude::*;

use crate::game::assets::Assets;

use super::{
    polygon::trimesh_from_polygon,
    svg::{read_svg, SvgShape},
};

pub fn shape_to_collider(shape: &SvgShape) -> (Isometry2<f32>, SharedShape) {
    match shape {
        SvgShape::Rect(rect) => {
            let half_dims = rect.dims / 2.0;
            (
                Isometry::from_parts(rect.pos.into(), UnitComplex::from_angle(rect.rotate)),
                SharedShape::cuboid(half_dims.x, half_dims.y),
            )
        }
        SvgShape::Circle(circle) => {
            // we don't care about the rotation of the circle
            (circle.pos.into(), SharedShape::ball(circle.r))
        }
        SvgShape::Path(path) => {
            let vertices = &path.vertices;
            let indices = trimesh_from_polygon(&vertices);
            let vertices = vertices.iter().map(|&v| v.into()).collect::<Vec<_>>();
            (Vec2::ZERO.into(), SharedShape::trimesh(vertices, indices))
        }
    }
}

pub fn load_collider(assets: &Assets, collider: &str) -> (Rect, ColliderBuilder) {
    let svg = &assets.colliders[collider];
    let (size, items) = read_svg(&svg);
    let translate = -size / 2.0;
    let shapes = items
        .into_iter()
        .map(|item| shape_to_collider(&item.shape))
        .map(|(pos, shape)| (pos.into(), shape))
        .collect::<Vec<_>>();
    let builder = ColliderBuilder::compound(shapes).translation(translate.into());
    let rect = Rect::new(-size.x / 2.0, -size.y / 2.0, size.x, size.y);
    (rect, builder)
}
