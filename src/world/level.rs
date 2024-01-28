use std::f32::consts::PI;

use crate::draw::pixel_to_meter;
use crate::{polygon::vertices_to_clockwise, polygon::OrderedVec2, world::floor::Material};

use super::floor::{spawn_floor, spawn_thing};
use super::World;
use indexmap::IndexSet;
use macroquad::prelude::*;
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    namespace::Namespace,
    reader::{EventReader, XmlEvent},
};

include!(concat!(env!("OUT_DIR"), "/level_codegen.rs"));

type StartElementEvent = (OwnedName, Vec<OwnedAttribute>, Namespace);
pub fn load_level(level: usize, world: &mut World) {
    println!("Loading level {}", level);
    let svg = LEVEL_SVGS[level];
    let reader = EventReader::from_str(svg);
    let mut floors: Vec<(Vec<Vec2>, Material)> = Vec::new();
    let mut things: Vec<(Vec2, Vec2, f32, Material)> = Vec::new();
    let mut svg_start_tag: Option<StartElementEvent> = None;
    for e in reader {
        match e {
            Ok(XmlEvent::StartElement {
                ref name,
                ref attributes,
                ref namespace,
            }) => match name.local_name.as_str() {
                "svg" => {
                    svg_start_tag = Some((name.clone(), attributes.clone(), namespace.clone()));
                }
                "path" => {
                    let vertices = path_to_vertices(
                        svg_start_tag.clone().expect("no starting svg tag"),
                        (name.clone(), attributes.clone(), namespace.clone()),
                    );
                    let kind = Material::from_hex_color(
                        u32::from_str_radix(
                            attributes
                                .iter()
                                .find(|a| a.name.local_name == "fill")
                                .expect("no fill attribute")
                                .value
                                .trim_start_matches("#"),
                            16,
                        )
                        .expect("fill attribute is not a hex color"),
                    );
                    floors.push((vertices, kind));
                }
                "rect" => {
                    let (pos, dims, rotate) = svg_rect_to_rect(attributes);
                    let kind = Material::from_hex_color(
                        u32::from_str_radix(
                            attributes
                                .iter()
                                .find(|a| a.name.local_name == "fill")
                                .expect("no fill attribute")
                                .value
                                .trim_start_matches("#"),
                            16,
                        )
                        .expect("fill attribute is not a hex color"),
                    );
                    things.push((pos, dims, rotate, kind));
                }
                _ => {}
            },
            Err(e) => {
                panic!("Error: {}", e);
            }
            _ => {}
        }
    }

    for (vertices, material) in floors {
        spawn_floor(world, vertices, material);
    }
    for (pos, dims, rotate, material) in things {
        spawn_thing(world, pos, dims, rotate, material);
    }
}

pub fn svg_rect_to_rect(attributes: &[OwnedAttribute]) -> (Vec2, Vec2, f32) {
    let get_attr = |name: &str| {
        &attributes
            .iter()
            .find(|a| a.name.local_name == name)
            .expect(&format! {"no {} attribute", name})
            .value
    };
    let parse_attr = |name: &str| {
        get_attr(name)
            .parse::<f32>()
            .expect(&format! {"attribute {} is not a number", name})
    };
    let x = parse_attr("x");
    let y = parse_attr("y");
    let width = parse_attr("width");
    let height = parse_attr("height");
    let rotate = {
        let rotate = get_attr("transform");
        let rotate = rotate.trim_start_matches("rotate(");
        let rotate = rotate.trim_end_matches(")");
        let (rotate, _) = rotate
            .split_once(" ")
            .expect("no space in rotate attribute");
        let rotate = rotate
            .parse::<f32>()
            .expect("rotate attribute is not a number");
        ((rotate / 360.0) * 2.0 * PI).rem_euclid(2.0 * PI)
    };
    let pos = Vec2::new(x, y);
    let translate_1 = Vec2::new(rotate.cos(), rotate.sin()) * (width / 2.0);
    let translate_2 =
        Vec2::new(rotate.cos(), rotate.sin()).rotate(Vec2::new(0.0, 1.0)) * (height / 2.0);
    let pos = pos + translate_1 + translate_2;
    (
        pixel_to_meter(pos),
        pixel_to_meter(Vec2::new(width, height) * 0.5),
        rotate,
    )
}

use svg2polylines;
use xml::writer::{EmitterConfig, EventWriter};
pub fn path_to_vertices(
    svg_start_tag: StartElementEvent,
    path_start_tag: StartElementEvent,
) -> Vec<Vec2> {
    fn start_end_element(
        name: OwnedName,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> (XmlEvent, XmlEvent) {
        let start = XmlEvent::StartElement {
            name: name.clone(),
            attributes,
            namespace,
        };
        let end = XmlEvent::EndElement { name };
        (start, end)
    }
    let mut output = Vec::new();
    let mut writer = EventWriter::new_with_config(&mut output, EmitterConfig::default());

    let (svg_start, svg_end) = start_end_element(svg_start_tag.0, svg_start_tag.1, svg_start_tag.2);
    let (path_start, path_end) =
        start_end_element(path_start_tag.0, path_start_tag.1, path_start_tag.2);

    writer.write(svg_start.as_writer_event().unwrap()).unwrap();
    writer.write(path_start.as_writer_event().unwrap()).unwrap();
    writer.write(path_end.as_writer_event().unwrap()).unwrap();
    writer.write(svg_end.as_writer_event().unwrap()).unwrap();

    let svg = String::from_utf8(output).unwrap();

    let polylines = svg2polylines::parse(&svg, 15.0, true).unwrap();

    assert_eq!(polylines.len(), 1);

    let polyline = &polylines[0];
    let vertices: Vec<Vec2> = polyline
        .iter()
        .map(|p| vec2(p.x as f32, p.y as f32))
        .map(pixel_to_meter)
        .collect();
    let vertices_set: IndexSet<OrderedVec2> = vertices.iter().map(|&v| v.into()).collect();
    let vertices = vertices_set.into_iter().map(|v| v.into()).collect();
    vertices_to_clockwise(vertices)
}
