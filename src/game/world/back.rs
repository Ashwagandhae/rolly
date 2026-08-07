use crate::{consts::BACKGROUND_TRANSITION_SPEED, game::world::level::LevelId};

pub struct Back {
    pub fade: f32,
    pub previous: Option<LevelId>,
    pub target: LevelId,
}
impl Back {
    pub fn new(level: LevelId) -> Self {
        Self {
            fade: 1.0,
            previous: None,
            target: level,
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.fade += BACKGROUND_TRANSITION_SPEED * dt;
        self.fade = self.fade.clamp(0.0, 1.0);
    }
    pub fn set_target(&mut self, new_target: LevelId) {
        if self.target == new_target {
            return;
        }
        self.fade = if self.previous.is_some_and(|prev| prev == new_target) {
            1.0 - self.fade
        } else {
            0.0
        };
        self.previous = Some(self.target);
        self.target = new_target;
    }
    pub fn render(&self) -> (LevelId, f32, LevelId) {
        (self.previous.unwrap_or(self.target), self.fade, self.target)
    }
}
