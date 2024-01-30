use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use macroquad::ui::{
    hash, root_ui,
    widgets::{self},
    Skin,
};
use macroquad::ui::{Layout, Ui};

use super::world::World;
use super::{Game, Screen};

const MARGIN: f32 = 10.0;
const ITEM_WIDTH: f32 = 200.0;
const ITEM_HEIGHT: f32 = 60.0;

fn skin() -> Skin {
    let button_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(255, 255, 255, 64))
        .color_hovered(Color::from_rgba(255, 255, 255, 96))
        .color_clicked(Color::from_rgba(255, 255, 255, 128))
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .text_color_hovered(Color::from_rgba(255, 255, 255, 255))
        .text_color_clicked(Color::from_rgba(255, 255, 255, 255))
        .font_size(25)
        .build();
    let label_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(255, 255, 255, 64))
        .color_hovered(Color::from_rgba(255, 255, 255, 96))
        .color_clicked(Color::from_rgba(255, 255, 255, 128))
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .text_color_hovered(Color::from_rgba(255, 255, 255, 255))
        .text_color_clicked(Color::from_rgba(255, 255, 255, 255))
        .font_size(25)
        .build();
    let editbox_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(255, 255, 255, 64))
        .color_hovered(Color::from_rgba(255, 255, 255, 96))
        .color_clicked(Color::from_rgba(255, 255, 255, 128))
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .text_color_hovered(Color::from_rgba(255, 255, 255, 255))
        .text_color_clicked(Color::from_rgba(255, 255, 255, 255))
        .font_size(15)
        .build();
    let window_style = root_ui()
        .style_builder()
        .color(Color::from_rgba(0, 0, 0, 96))
        .margin(RectOffset::new(MARGIN, MARGIN, MARGIN, MARGIN))
        .build();
    Skin {
        button_style,
        window_style,
        editbox_style,
        label_style,
        ..root_ui().default_skin()
    }
}

fn draw_buttons(mut game: &mut Game, ui: &mut Ui, buttons: &[(&str, fn(&mut Game))]) {
    let mut y = 0.0;

    for (text, action) in buttons {
        if widgets::Button::new(*text)
            .position(vec2(0.0, y))
            .size(vec2(ITEM_WIDTH, ITEM_HEIGHT))
            .ui(ui)
        {
            action(&mut game);
        }
        y += ITEM_HEIGHT + MARGIN;
    }
}

fn basic_window(f: impl FnOnce(&mut Ui)) {
    root_ui().window(
        hash!("window", (screen_width() + screen_height()) as usize),
        vec2(0., 0.),
        vec2(ITEM_WIDTH + MARGIN * 2.0, screen_height()),
        f,
    );
}

pub fn tick(game: &mut Game) {
    let skin = skin();
    root_ui().push_skin(&skin);
    match game.screen {
        Screen::Home => home(game),
        Screen::Settings => settings(game, false),
        Screen::SettingsPaused => settings(game, true),
        Screen::Paused => paused(game),
        Screen::Running => running(game),
        Screen::Quit => quit(game),
    }
}

fn home(game: &mut Game) {
    basic_window(|ui| {
        draw_buttons(
            game,
            ui,
            &[
                ("New World", |game| {
                    new_world(game);
                    game.screen = Screen::Running;
                }),
                ("Settings", |game| change_screen(game, Screen::Settings)),
                ("Quit", |game| change_screen(game, Screen::Quit)),
            ],
        )
    });
}

fn new_world(game: &mut Game) {
    game.world = Some(World::new(&game.texture_holder));
}

fn settings(game: &mut Game, paused: bool) {
    basic_window(|ui| {
        // widgets::Slider::new(
        //     &mut game.settings.volume,
        //     0.0..=1.0,
        // )
        // .position(vec2(0.0, 0.0))
        const SETTINGS_ITEM_COUNT: f32 = 3.0;

        widgets::Group::new(
            hash!(),
            vec2(
                ITEM_WIDTH + MARGIN * 2.0,
                (ITEM_HEIGHT + MARGIN) * SETTINGS_ITEM_COUNT,
            ),
        )
        .ui(ui, |ui| {
            widgets::Slider::new(hash!(), 0.0..1.0).ui(ui, &mut game.settings.volume);
        });
        widgets::Group::new(
            hash!(),
            vec2(ITEM_WIDTH + MARGIN * 2.0, (ITEM_HEIGHT + MARGIN) * 1.0),
        )
        .ui(ui, |ui| {
            draw_buttons(
                game,
                ui,
                &[if paused {
                    ("Back", |game| change_screen(game, Screen::Paused))
                } else {
                    ("Home", |game| change_screen(game, Screen::Home))
                }],
            );
        });
    });
}

fn paused(game: &mut Game) {
    if is_key_pressed(KeyCode::Escape) {
        change_screen(game, Screen::Running);
    }
    basic_window(|ui| {
        draw_buttons(
            game,
            ui,
            &[
                ("Resume", |game| change_screen(game, Screen::Running)),
                ("Settings", |game| {
                    change_screen(game, Screen::SettingsPaused)
                }),
                ("Save", |game| save_world(game)),
                ("Home", |game| change_screen(game, Screen::Home)),
            ],
        );
    });
}

fn change_screen(game: &mut Game, screen: Screen) {
    match (game.screen, screen) {
        (Screen::Running | Screen::Paused, Screen::Home | Screen::Quit) => {
            save_world(game);
            game.world = None;
        }
        _ => {}
    }
    game.screen = screen;
}

fn running(game: &mut Game) {
    if is_key_pressed(KeyCode::Escape) {
        game.screen = Screen::Paused;
    }
}

fn quit(_game: &mut Game) {}

fn save_world(_game: &mut Game) {}

// pub struct Game {
//     texture_holder: TextureHolder,
//     storage: Storage {
//         settings: Settings {
//             volume: f32,
//             zoom: bool,
//         },
//         save: Save,
//    },
//     screen: pub enum Screen {
//          Home,
//          Settings,
//          Paused,
//          Running,
//     }
//     world: World {
//         player: Player,
//         entities: HecsWorld,
//         camera: Camera2D,
//         zoom: f32,
//         physics_world: PhysicsWorld,
//     }
// }
// Home
// new game
// load game
// settings
// quit
// Paused
// resume
// save game
// home
// quit
// Running
// pause
