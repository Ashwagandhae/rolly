use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::game::world::draw::pixel_to_meter;
use crate::game::world::polygon::{vertices_to_clockwise, OrderedVec2};

use indexmap::IndexSet;
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    namespace::Namespace,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, Clone)]
pub enum SvgShape {
    Rect(RectShape),
    Path(PathShape),
    Circle(CircleShape),
}

#[derive(Debug, Clone)]
pub struct RectShape {
    /// center pos
    pub pos: Vec2,
    /// width, height
    pub dims: Vec2,
    pub rotate: f32,
}

#[derive(Debug, Clone)]
pub struct PathShape {
    pub vertices: Vec<Vec2>,
}

#[derive(Debug, Clone)]
pub struct CircleShape {
    pub pos: Vec2,
    pub r: f32,
    pub rotate: f32,
}

#[derive(Debug, Clone)]
pub struct SvgItem {
    pub shape: SvgShape,
    pub color: u32,
    pub index: usize,
}

type StartElementEvent = (OwnedName, Vec<OwnedAttribute>, Namespace);
pub fn read_svg(svg: &str) -> (Vec2, Vec<SvgItem>) {
    let mut reader = EventReader::from_str(svg).into_iter();
    let svg_start_tag = reader
        .find_map(|e| match e {
            Ok(XmlEvent::StartElement {
                ref name,
                ref attributes,
                ref namespace,
            }) if name.local_name == "svg" => {
                Some((name.clone(), attributes.clone(), namespace.clone()))
            }
            _ => None,
        })
        .expect("no svg tag");
    let width = parse_attr("width", &svg_start_tag.1).expect("no width attribute");
    let height = parse_attr("height", &svg_start_tag.1).expect("no height attribute");
    let mut items = Vec::new();
    let mut index = 0;
    for event in reader {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "clipPath" => break,
            Ok(XmlEvent::StartElement {
                ref name,
                ref attributes,
                ref namespace,
            }) if name.local_name == "rect"
                || name.local_name == "path"
                || name.local_name == "circle" =>
            {
                let color = u32::from_str_radix(
                    attributes
                        .iter()
                        .find(|a| a.name.local_name == "fill")
                        .expect("no fill attribute")
                        .value
                        .trim_start_matches("#"),
                    16,
                )
                .expect("fill attribute is not a hex color");
                let shape = match name.local_name.as_str() {
                    "rect" => SvgShape::Rect(read_svg_rect(attributes)),
                    "path" => SvgShape::Path(read_svg_path(
                        svg_start_tag.clone(),
                        (name.clone(), attributes.clone(), namespace.clone()),
                    )),
                    "circle" => SvgShape::Circle(read_svg_circle(attributes)),
                    _ => panic!("unknown shape {} in svg", name.local_name),
                };
                items.push(SvgItem {
                    shape,
                    color,
                    index,
                });
                index += 1;
            }
            _ => {}
        }
    }
    (pixel_to_meter(vec2(width, height)), items)
}

fn get_attr<'a>(attributes: &'a [OwnedAttribute], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.as_str())
}
fn parse_attr(name: &str, attributes: &[OwnedAttribute]) -> Option<f32> {
    get_attr(attributes, name).map(|s| s.parse::<f32>().unwrap())
}
fn get_rotate(attributes: &[OwnedAttribute]) -> f32 {
    let Some(rotate) = &attributes
        .iter()
        .find(|a| a.name.local_name == "transform")
        .map(|a| &a.value)
    else {
        return 0.0;
    };
    let rotate = rotate.trim_start_matches("rotate(");
    let rotate = rotate.trim_end_matches(")");
    let (rotate, _) = rotate
        .split_once(" ")
        .expect("no space in rotate attribute");
    let rotate = rotate
        .parse::<f32>()
        .expect("rotate attribute is not a number");
    ((rotate / 360.0) * 2.0 * PI).rem_euclid(2.0 * PI)
}
pub fn read_svg_rect(attributes: &[OwnedAttribute]) -> RectShape {
    let parse_attr = |name| parse_attr(name, attributes);

    let x = parse_attr("x").expect("no x attribute");
    let y = parse_attr("y").expect("no y attribute");
    let width = parse_attr("width").expect("no width attribute");
    let height = parse_attr("height").expect("no height attribute");
    let rotate = get_rotate(attributes);

    let pos = Vec2::new(x, y);
    let translate_1 = Vec2::new(rotate.cos(), rotate.sin()) * (width / 2.0);
    let translate_2 =
        Vec2::new(rotate.cos(), rotate.sin()).rotate(Vec2::new(0.0, 1.0)) * (height / 2.0);
    let pos = pos + translate_1 + translate_2;
    RectShape {
        pos: pixel_to_meter(pos),
        dims: pixel_to_meter(Vec2::new(width, height)),
        rotate,
    }
}

pub fn read_svg_circle(attributes: &[OwnedAttribute]) -> CircleShape {
    let parse_attr = |name| parse_attr(name, attributes);

    let cx = parse_attr("cx").unwrap_or(0.0);
    let cy = parse_attr("cy").unwrap_or(0.0);
    let r = parse_attr("r").expect("no r attribute");
    let rotate = get_rotate(attributes);

    let pos = Vec2::new(cx, cy);
    CircleShape {
        pos: pixel_to_meter(pos),
        r: pixel_to_meter(r),
        rotate,
    }
}

use svg2polylines;
use xml::writer::{EmitterConfig, EventWriter};
pub fn read_svg_path(
    svg_start_tag: StartElementEvent,
    path_start_tag: StartElementEvent,
) -> PathShape {
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
    let vertices = vertices_to_clockwise(vertices);
    PathShape { vertices }
}
