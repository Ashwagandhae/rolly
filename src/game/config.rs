#[cfg(not(target_arch = "wasm32"))]
use argh::FromArgs;

#[cfg(not(target_arch = "wasm32"))]
#[derive(FromArgs)]
/// Game configuration
pub struct GameConfig {
    #[argh(switch)]
    /// whether or not cheats are on
    pub(crate) cheat: bool,
    #[argh(option)]
    /// level to start the game on
    pub(crate) level: Option<usize>,
}

#[cfg(target_arch = "wasm32")]
/// Game configuration
pub struct GameConfig {
    /// whether or not cheats are on
    pub(crate) cheat: bool,
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
