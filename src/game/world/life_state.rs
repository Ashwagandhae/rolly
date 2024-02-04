use super::frame::Transition;

#[derive(Debug, Clone)]
pub enum LifeState {
    Alive(Transition),
    Dead(Transition),
}
