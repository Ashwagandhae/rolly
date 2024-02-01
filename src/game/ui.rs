use egui_macroquad::egui::FontFamily::Proportional;
use egui_macroquad::egui::{Context, FontId, TextStyle};
use egui_macroquad::{egui, egui::Ui};
use macroquad::prelude::*;

pub mod settings;

use self::settings::SettingKindMut;

use super::world::World;
use super::{save_world, Game, Screen};
use settings::{Setting, SettingInfo, Settings, Slider, Toggle};

const MARGIN: f32 = 10.0;
const ITEM_WIDTH: f32 = 200.0;
const ITEM_HEIGHT: f32 = 0.0;
pub fn update_ui_scale(ctx: &Context, settings: &Settings) {
    ctx.set_pixels_per_point(
        match settings.ui_scale.info.options[settings.ui_scale.value] {
            ".25x" => 0.25,
            ".5x" => 0.5,
            "1x" => 1.0,
            "2x" => 2.0,
            "4x" => 4.0,
            "6x" => 6.0,
            _ => 1.0,
        },
    );
}
pub fn init(settings: &Settings) {
    egui_macroquad::ui(|egui_ctx| {
        let mut style = (*egui_ctx.style()).clone();

        style.spacing.item_spacing = egui::Vec2::new(MARGIN, MARGIN);
        style.spacing.button_padding = egui::Vec2::new(MARGIN, MARGIN);
        style.spacing.slider_width = ITEM_WIDTH - 70.0;
        style.spacing.icon_width = 25.0;
        style.spacing.icon_width_inner = 15.0;

        // create new font family
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "JetBrains".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/JetBrainsMono-Regular.ttf"
            )),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "JetBrains".to_owned());

        egui_ctx.set_fonts(fonts);

        *style.text_styles.get_mut(&TextStyle::Button).unwrap() = FontId::new(16.0, Proportional);
        *style.text_styles.get_mut(&TextStyle::Body).unwrap() = FontId::new(16.0, Proportional);

        style.visuals.widgets.inactive.weak_bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4);
        style.visuals.widgets.hovered.weak_bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 16);
        style.visuals.widgets.active.weak_bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 32);
        style.visuals.widgets.inactive.bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8);
        style.visuals.widgets.hovered.bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8);
        style.visuals.widgets.active.bg_fill =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8);

        style.visuals.popup_shadow.extrusion = 0.0;
        style.visuals.popup_shadow.color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0);

        style.visuals.widgets.inactive.rounding = MARGIN.into();
        style.visuals.widgets.hovered.rounding = MARGIN.into();
        style.visuals.widgets.active.rounding = MARGIN.into();
        basic_window(|ui| {
            ui.label("Loading...");
        });
        update_ui_scale(egui_ctx, settings);
        egui_ctx.set_style(style);
    });
    egui_macroquad::draw();
}

fn draw_buttons(mut game: &mut Game, ui: &mut Ui, buttons: &[(&str, fn(&mut Game))]) {
    for (text, action) in buttons {
        let button = egui::Button::new(*text).min_size(egui::Vec2::new(ITEM_WIDTH, ITEM_HEIGHT));
        if ui.add(button).clicked() {
            action(&mut game);
        }
    }
}

fn basic_window(f: impl FnOnce(&mut Ui)) {
    egui_macroquad::ui(|egui_ctx| {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .inner_margin(MARGIN)
                    .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 200)),
            )
            .show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), f)
                });
            });
    });
    egui_macroquad::draw();
}

pub fn tick(game: &mut Game) {
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
                ("new world", |game| {
                    new_world(game);
                    game.screen = Screen::Running;
                }),
                ("settings", |game| change_screen(game, Screen::Settings)),
                ("quit", |game| change_screen(game, Screen::Quit)),
            ],
        )
    });
}

fn new_world(game: &mut Game) {
    game.world = Some(World::new(&game.texture_holder));
}

fn draw_label(ui: &mut Ui, setting: &Setting<impl SettingInfo>) {
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(0.0, MARGIN))
        .show(ui, |ui| ui.add(egui::Label::new(setting.name)));
}
fn draw_reset_button(ui: &mut Ui, setting: &mut Setting<impl SettingInfo>) {
    if setting.value != setting.info.default_value() {
        if ui.add(egui::Button::new("reset")).clicked() {
            setting.value = setting.info.default_value();
        }
    }
}

fn draw_slider(ui: &mut Ui, setting: &mut Setting<Slider>) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                draw_label(ui, setting);
                draw_reset_button(ui, setting);
            });

            ui.add(egui::Slider::new(
                &mut setting.value,
                setting.info.range.clone(),
            ));
        });
    });
}

fn draw_toggle(ui: &mut Ui, setting: &mut Setting<Toggle>) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            draw_label(ui, setting);
            ui.add(toggle(&mut setting.value));
            draw_reset_button(ui, setting);
        });
    });
}

fn draw_combo_box(ui: &mut Ui, setting: &mut Setting<settings::ComboBox>) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                draw_label(ui, setting);
                draw_reset_button(ui, setting);
            });
            egui::ComboBox::from_label("")
                .selected_text(setting.info.options[setting.value])
                .show_index(ui, &mut setting.value, setting.info.options.len(), |i| {
                    setting.info.options[i].to_owned()
                });
        });
    });
}

/// Here is the same code again, but a bit more compact:
fn toggle_ui_compact(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

// A wrapper that allows the more idiomatic usage pattern: `ui.add(toggle(&mut my_bool))`
/// iOS-style toggle switch.
///
/// ## Example:
/// ``` ignore
/// ui.add(toggle(&mut my_bool));
/// ```
pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui_compact(ui, on)
}

fn settings(game: &mut Game, paused: bool) {
    basic_window(|ui| {
        update_ui_scale(ui.ctx(), &game.settings);

        if is_key_pressed(KeyCode::Escape) {
            if paused {
                change_screen(game, Screen::Paused);
            } else {
                change_screen(game, Screen::Home);
            }
        }
        for setting in &mut game.settings.iter_mut() {
            match setting {
                SettingKindMut::Slider(setting) => draw_slider(ui, setting),
                SettingKindMut::Toggle(setting) => draw_toggle(ui, setting),
                SettingKindMut::ComboBox(setting) => draw_combo_box(ui, setting),
            }
        }
        draw_buttons(
            game,
            ui,
            &[if paused {
                ("back", |game| change_screen(game, Screen::Paused))
            } else {
                ("back", |game| change_screen(game, Screen::Home))
            }],
        );
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
                ("resume", |game| change_screen(game, Screen::Running)),
                ("settings", |game| {
                    change_screen(game, Screen::SettingsPaused)
                }),
                ("save", |game| save_world(game)),
                ("save & back", |game| change_screen(game, Screen::Home)),
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

fn constrain_slider(setting: &mut Setting<Slider>) {
    setting.value = setting
        .value
        .clamp(*setting.info.range.start(), *setting.info.range.end());
}

fn running(game: &mut Game) {
    if is_key_pressed(KeyCode::Escape) {
        game.screen = Screen::Paused;
    }
    let settings = &mut game.settings;
    if is_key_down(KeyCode::Equal) {
        settings.zoom.value *= 1.01;
    }
    if is_key_down(KeyCode::Minus) {
        settings.zoom.value *= 0.99;
    }
    if is_key_pressed(KeyCode::Key0) {
        settings.zoom.value = 1.0;
    }
    constrain_slider(&mut settings.zoom);
}

fn quit(_game: &mut Game) {}

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
