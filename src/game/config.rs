use std::default::Default;

#[cfg(not(target_arch = "wasm32"))]
use argh::FromArgs;

#[cfg_attr(not(target_arch = "wasm32"), derive(FromArgs))]
/// Game configuration
pub struct GameConfig {
    #[cfg_attr(not(target_arch = "wasm32"), argh(switch))]
    /// whether or not cheats are on
    pub(crate) cheat: bool,

    #[cfg_attr(not(target_arch = "wasm32"), argh(option))]
    /// level to start the game on
    pub(crate) level: Option<usize>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            cheat: false,
            level: None,
        }
    }
}
