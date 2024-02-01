use std::ops::RangeInclusive;

mod macros;

use macros::make_settings_struct;

pub trait SettingInfo {
    type Value: Clone + PartialEq;
    fn default_value(&self) -> Self::Value;
}

pub struct Setting<T: SettingInfo + 'static> {
    pub name: &'static str,
    pub value: T::Value,
    pub info: &'static T,
}

pub struct Slider {
    pub default: f32,
    pub range: RangeInclusive<f32>,
}

impl Slider {
    const fn from_tuple((default, range): (f32, RangeInclusive<f32>)) -> Self {
        Self { default, range }
    }
}
impl SettingInfo for Slider {
    type Value = f32;
    fn default_value(&self) -> Self::Value {
        self.default
    }
}
pub struct Toggle {
    pub default: bool,
}

impl Toggle {
    const fn from_tuple(default: bool) -> Self {
        Self { default }
    }
}
impl SettingInfo for Toggle {
    type Value = bool;
    fn default_value(&self) -> Self::Value {
        self.default
    }
}

pub struct ComboBox {
    pub default: usize,
    pub options: &'static [&'static str],
}

impl ComboBox {
    const fn from_tuple((default, options): (usize, &'static [&'static str])) -> Self {
        Self { default, options }
    }
}

impl SettingInfo for ComboBox {
    type Value = usize;
    fn default_value(&self) -> Self::Value {
        self.default
    }
}

pub enum SettingKind<'a> {
    Slider(&'a Setting<Slider>),
    Toggle(&'a Setting<Toggle>),
    ComboBox(&'a Setting<ComboBox>),
}

pub enum SettingKindMut<'a> {
    Slider(&'a mut Setting<Slider>),
    Toggle(&'a mut Setting<Toggle>),
    ComboBox(&'a mut Setting<ComboBox>),
}

make_settings_struct!(pub Settings {
    ui_scale: "ui scale", ComboBox, (3, &[".25x", ".5x", "1x", "2x", "4x", "6x"]),
    zoom: "zoom", Slider, (1.0, 0.25..=4.0),
    camera_speed: "camera speed", Slider, (0.1, 0.01..=1.0),
    fullscreen: "fullscreen", Toggle, false,
});
