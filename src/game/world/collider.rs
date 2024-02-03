use macroquad::prelude::*;
use rapier2d::prelude::*;

use crate::game::assets::Assets;

use super::{
    polygon::trimesh_indices_from_polygon,
    svg::{read_svg, SvgShape},
};

pub fn shape_to_collider(shape: &SvgShape) -> (Vec2, SharedShape) {
    match shape {
        SvgShape::Rect(rect) => {
            assert!(rect.rotate == 0.0, "rotated rects not supported");
            let half_dims = rect.dims / 2.0;
            (rect.pos, SharedShape::cuboid(half_dims.x, half_dims.y))
        }
        SvgShape::Circle(circle) => {
            // we don't care about the rotation of the circle
            (circle.pos, SharedShape::ball(circle.r))
        }
        SvgShape::Path(path) => {
            let vertices = &path.vertices;
            let indices = trimesh_indices_from_polygon(&vertices);
            let vertices = vertices.iter().map(|&v| v.into()).collect::<Vec<_>>();
            (Vec2::ZERO, SharedShape::trimesh(vertices, indices))
        }
    }
}

pub fn load_collider(assets: &Assets, collider: &str) -> ColliderBuilder {
    let svg = &assets.colliders[collider];
    let (size, items) = read_svg(&svg);
    let translate = -size / 2.0;
    let shapes = items
        .into_iter()
        .map(|item| shape_to_collider(&item.shape))
        .map(|(pos, shape)| (pos.into(), shape))
        .collect::<Vec<_>>();
    let builder = ColliderBuilder::compound(shapes).translation(translate.into());
    builder
}
