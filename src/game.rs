pub mod texture;
pub mod ui;
pub mod world;

use self::texture::TextureHolder;
use macroquad::prelude::*;

use world::{draw::draw as draw_world, update::update as update_world, World};

pub struct Game {
    pub texture_holder: TextureHolder,
    pub settings: Settings,
    pub screen: Screen,
    pub world: Option<World>,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            texture_holder: TextureHolder::new().await,
            settings: Settings::new(),
            screen: Screen::Home,
            world: None,
        }
    }
    pub fn quit(&self) -> bool {
        if let Screen::Quit = self.screen {
            true
        } else {
            false
        }
    }
}

pub fn tick(game: &mut Game) {
    if let Some(world) = &mut game.world {
        draw_world(&game.texture_holder, world);
        if let Screen::Running = game.screen {
            update_world(world);
        }
    }
    ui::tick(game);
}

pub struct Settings {
    pub volume: f32,
    pub zoom: f32,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            zoom: 1.0,
        }
    }
}

pub struct SavedWorld {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Settings,
    SettingsPaused,
    Paused,
    Running,
    Quit,
}
