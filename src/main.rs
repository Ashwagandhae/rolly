use macroquad::prelude::*;
pub mod consts;
pub mod game;

use game::{tick, Game};

use crate::game::config::GameConfig;

fn window_conf() -> Conf {
    Conf {
        window_title: "rolly polly".to_owned(),
        high_dpi: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}
fn get_config() -> GameConfig {
    #[cfg(not(target_arch = "wasm32"))]
    {
        argh::from_env()
    }
    #[cfg(target_arch = "wasm32")]
    {
        GameConfig::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(get_config()).await;
    loop {
        tick(&mut game);
        next_frame().await;
        if game.quit() {
            break;
        }
    }
}
