use macroquad::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use std::ops::RangeInclusive;

use super::world::level::LevelInfo;

type SizedTexture = ((usize, usize), Texture2D);
pub struct Assets {
    pub textures: HashMap<String, SizedTexture>,
    pub tiles: HashMap<String, Tile>,
    pub colliders: HashMap<String, String>,
    pub levels: HashMap<usize, (LevelInfo, String)>,
}

include!(concat!(env!("OUT_DIR"), "/asset_codegen.rs"));

impl Assets {
    pub async fn new() -> Self {
        let texture_paths = TEXTURE_FILENAMES
            .iter()
            .map(|(filename, _)| format!("assets/textures/{}", filename))
            .collect::<Vec<_>>();
        let mut texture_load = Vec::new();
        for filename in texture_paths {
            texture_load.push(load_texture(&filename).await);
        }
        let textures = texture_load.into_iter().map(|texture| {
            let texture = texture.unwrap();
            texture.set_filter(FilterMode::Nearest);
            texture
        });
        let texture_name_and_args = TEXTURE_FILENAMES
            .iter()
            .map(|(filename, size)| {
                let filename = filename.strip_suffix(".png").expect("expected png image");
                let (name, args) = split_name_args(filename);
                (name, *size, args)
            })
            .collect::<Vec<_>>();
        let tiles = extract_tiles(&texture_name_and_args);
        let textures = texture_name_and_args
            .iter()
            .zip(textures)
            .map(|((name, size, _), texture)| (name.to_string(), (*size, texture)))
            .collect::<HashMap<_, _>>();

        let collider_paths = COLLIDER_FILENAMES
            .iter()
            .map(|filename| format!("assets/colliders/{}", filename))
            .collect::<Vec<_>>();
        let mut colliders_load = Vec::new();
        for filename in collider_paths {
            colliders_load.push(load_string(&filename).await);
        }
        let colliders = COLLIDER_FILENAMES
            .iter()
            .map(|filename| {
                let filename = filename.strip_suffix(".svg").expect("expected svg file");
                let (name, _) = split_name_args(filename);
                name.to_string()
            })
            .zip(colliders_load.into_iter().map(|collider| collider.unwrap()))
            .collect::<HashMap<_, _>>();

        let level_paths = LEVEL_FILENAMES
            .iter()
            .map(|filename| format!("assets/levels/{}", filename))
            .collect::<Vec<_>>();
        let mut levels_load = Vec::new();
        for filename in level_paths {
            levels_load.push(load_string(&filename).await);
        }
        let levels = LEVEL_FILENAMES
            .iter()
            .map(|filename| {
                let filename = filename.strip_suffix(".svg").expect("expected svg file");
                let (name, _) = split_name_args(filename);
                name.parse().unwrap()
            })
            .zip(
                levels_load
                    .into_iter()
                    .map(|level| level.unwrap())
                    .map(|level| {
                        let level_info = LevelInfo::parse(&level);
                        (level_info, level)
                    }),
            )
            .collect::<HashMap<_, _>>();
        Self {
            textures,
            tiles,
            colliders,
            levels,
        }
    }
}

impl Assets {
    pub fn get(&self, index: &str) -> Option<&SizedTexture> {
        self.textures.get(index)
    }
    pub fn get_mut(&mut self, index: &str) -> Option<&mut SizedTexture> {
        self.textures.get_mut(index)
    }
}

impl std::ops::Index<&str> for Assets {
    type Output = SizedTexture;

    fn index(&self, index: &str) -> &Self::Output {
        self.textures.get(index).unwrap()
    }
}

impl std::ops::IndexMut<&str> for Assets {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.textures.get_mut(index).unwrap()
    }
}

pub fn extract_tiles(
    name_and_args: &Vec<(&str, (usize, usize), Option<&str>)>,
) -> HashMap<String, Tile> {
    let tiles = name_and_args
        .iter()
        .filter_map(|(full_name, _, args)| {
            full_name.strip_prefix("tile_").map(|name| {
                let tile_name = name.split_once('_').map(|(name, _)| name).unwrap_or(name);
                let constraints = args
                    .map(|args| TileConstraints::parse(args))
                    .unwrap_or_default();
                (tile_name, (full_name, constraints))
            })
        })
        .fold(
            HashMap::<String, Tile>::default(),
            |mut tiles, (tile_name, (name, constraints))| {
                tiles
                    .entry(tile_name.to_string())
                    .or_insert(Tile(Vec::new()))
                    .0
                    .push((name.to_string(), constraints));
                tiles
            },
        );
    tiles
}

pub fn split_name_args(name: &str) -> (&str, Option<&str>) {
    name.split_once('(')
        .map(|(name, args)| {
            let args = args.strip_suffix(')').unwrap_or("expected end parantheses");
            (name, Some(args))
        })
        .unwrap_or((name, None))
}

#[derive(Debug, Clone)]
pub struct Tile(pub Vec<(String, TileConstraints)>);

#[derive(Debug, Clone)]
pub struct TileConstraints {
    pub left_height: RangeInclusive<u8>,
    pub height: u8,
    pub right_height: RangeInclusive<u8>,
    pub weight: u8,
}

impl std::default::Default for TileConstraints {
    fn default() -> Self {
        Self {
            left_height: 0..=1,
            height: 1,
            right_height: 0..=1,
            weight: 5,
        }
    }
}

impl TileConstraints {
    pub fn zero() -> Self {
        Self {
            left_height: 0..=255,
            height: 0,
            right_height: 0..=255,
            weight: 0,
        }
    }
    pub fn new_or_default(
        left_height: Option<RangeInclusive<u8>>,
        height: Option<u8>,
        right_height: Option<RangeInclusive<u8>>,
        weight: Option<u8>,
    ) -> Self {
        let default = Self::default();
        Self {
            left_height: left_height.unwrap_or(default.left_height),
            height: height.unwrap_or(default.height),
            right_height: right_height.unwrap_or(default.right_height),
            weight: weight.unwrap_or(default.weight),
        }
    }
    pub fn parse(constraints: &str) -> Self {
        fn parse_range(range: &str) -> RangeInclusive<u8> {
            if let Some((start, end)) = range.split_once('-') {
                return start.parse().unwrap()..=end.parse().unwrap();
            } else {
                return range.parse().unwrap()..=range.parse().unwrap();
            }
        }
        let constraint_items: Vec<&str> = constraints.split('_').collect();
        let left_height = constraint_items.get(0).map(|&s| parse_range(s));
        let height = constraint_items.get(1).map(|&s| s.parse().unwrap());
        let right_height = constraint_items.get(2).map(|&s| parse_range(s));
        let weight = constraint_items.get(3).map(|&s| s.parse().unwrap());
        Self::new_or_default(left_height, height, right_height, weight)
    }

    pub fn fits(&self, right: Self) -> bool {
        self.right_height.contains(&right.height) && right.left_height.contains(&self.height)
    }
}
