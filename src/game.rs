pub mod assets;
pub mod ui;
pub mod world;

use self::{assets::Assets, ui::settings::Settings};
use macroquad::prelude::*;

use world::{draw::draw as draw_world, update::update as update_world, World};

pub struct Game {
    pub assets: Assets,
    pub settings: Settings,
    pub screen: Screen,
    pub world: Option<World>,
}

impl Game {
    pub async fn new() -> Self {
        let settings = Settings::new();
        ui::init(&settings);
        Self {
            assets: Assets::new().await,
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
        if let Screen::Running = game.screen {
            update_world(&game.assets, &game.settings, world);
        }
        draw_world(&game.settings, &game.assets, world);
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
