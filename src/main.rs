use macroquad::prelude::*;

pub mod consts;
pub mod game;

use game::{tick, Game};

fn window_conf() -> Conf {
    Conf {
        window_title: "rolly polly".to_owned(),
        high_dpi: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;
    loop {
        tick(&mut game);
        next_frame().await;
        if game.quit() {
            break;
        }
    }
}
