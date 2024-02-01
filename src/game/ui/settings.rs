use std::ops::RangeInclusive;

mod macros;

use macros::make_settings_struct;

pub struct Slider {
    pub default: f32,
    pub range: RangeInclusive<f32>,
}

impl Slider {
    const fn from_tuple((default, range): (f32, RangeInclusive<f32>)) -> Self {
        Self { default, range }
    }
}

pub trait SettingInfo {
    type Value: Clone + PartialEq;
    fn default_value(&self) -> Self::Value;
    fn setting_with_name(&'static self, name: &'static str) -> Setting<Self>
    where
        Self: Sized,
    {
        Setting {
            name,
            value: self.default_value(),
            info: &self,
        }
    }
}
impl SettingInfo for Slider {
    type Value = f32;
    fn default_value(&self) -> Self::Value {
        self.default
    }
}

pub struct Setting<T: SettingInfo + 'static> {
    pub name: &'static str,
    pub value: T::Value,
    pub info: &'static T,
}
pub struct Toggle {
    pub default: bool,
}

impl Toggle {
    const fn from_tuple(default: bool) -> Self {
        Self { default }
    }
}

pub enum SettingKind<'a> {
    Slider(&'a Setting<Slider>),
    Toggle(&'a Setting<Toggle>),
}

pub enum SettingKindMut<'a> {
    Slider(&'a mut Setting<Slider>),
    Toggle(&'a mut Setting<Toggle>),
}

impl SettingInfo for Toggle {
    type Value = bool;
    fn default_value(&self) -> Self::Value {
        self.default
    }
}

make_settings_struct!(pub Settings {
    zoom: "zoom", Slider, (1.0, 0.25..=4.0),
    camera_speed: "camera speed", Slider, (0.1, 0.01..=1.0),
    fullscreen: "fullscreen", Toggle, false,
});
