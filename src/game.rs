pub mod texture;
pub mod ui;
pub mod world;

use self::{texture::TextureHolder, ui::settings::Settings};
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
        ui::init();
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
        draw_world(&game.settings, &game.texture_holder, world);
        if let Screen::Running = game.screen {
            update_world(&game.settings, world);
        }
    }
    // if requested to quit, save world
    if is_quit_requested() {
        save_world(game);
    }
    ui::tick(game);
}

fn save_world(_game: &mut Game) {}

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
