use macroquad::prelude::*;

mod consts;
mod draw;
mod update;
mod world;

use draw::draw;
use update::update;
use world::World;

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
    let mut world = World::new().await;
    loop {
        update(&mut world);
        draw(&world);

        next_frame().await
    }
}
