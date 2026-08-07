pub mod assets;
pub mod config;
pub mod ui;
pub mod world;

use crate::game::config::GameConfig;

use self::{assets::Assets, ui::settings::Settings};
use macroquad::prelude::*;

use world::{World, draw::draw as draw_world, update::update as update_world};

pub struct Game {
    pub assets: Assets,
    pub settings: Settings,
    pub screen: Screen,
    pub world: Option<World>,
    pub config: GameConfig,
}

impl Game {
    pub async fn new(config: GameConfig) -> Self {
        let settings = Settings::new();
        ui::init(&settings);
        Self {
            assets: Assets::new().await,
            settings: Settings::new(),
            screen: Screen::Home,
            world: None,
            config,
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

pub async fn tick(game: &mut Game) {
    if let Some(world) = &mut game.world {
        if let Screen::Running = game.screen {
            update_world(&mut game.assets, &game.settings, world, &game.config).await;
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
