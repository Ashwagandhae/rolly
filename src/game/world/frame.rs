#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousFrame(f32);

impl ContinuousFrame {
    pub fn new(val: f32) -> Self {
        Self(val.rem_euclid(1.0))
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

// implement add, sub, mul, div for ContinuousFrame
// so that we can do math with it

impl std::ops::Add<f32> for ContinuousFrame {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl std::ops::Sub<f32> for ContinuousFrame {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

impl std::ops::Mul<f32> for ContinuousFrame {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.0 * rhs)
    }
}

impl std::ops::Div<f32> for ContinuousFrame {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.0 / rhs)
    }
}

impl std::ops::AddAssign<f32> for ContinuousFrame {
    fn add_assign(&mut self, rhs: f32) {
        *self = Self::new(self.0 + rhs);
    }
}

impl std::ops::SubAssign<f32> for ContinuousFrame {
    fn sub_assign(&mut self, rhs: f32) {
        *self = Self::new(self.0 - rhs);
    }
}

impl std::ops::MulAssign<f32> for ContinuousFrame {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self::new(self.0 * rhs);
    }
}

impl std::ops::DivAssign<f32> for ContinuousFrame {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self::new(self.0 / rhs);
    }
}

#[derive(Debug, Clone)]
pub enum Transition {
    Start,
    End,
    Between {
        time: f32,
        duration: f32,
        forward: bool,
    },
}

impl Transition {
    pub fn tick(&mut self, delta_time: f32) {
        *self = match self.clone() {
            Self::Start => Self::Start,
            Self::End => Self::End,
            Self::Between {
                mut time,
                duration,
                forward,
            } => {
                time += delta_time / duration * if forward { -1.0 } else { 1.0 };
                if time < 0.0 {
                    Self::End
                } else if time > 1.0 {
                    Self::Start
                } else {
                    Self::Between {
                        time,
                        duration,
                        forward,
                    }
                }
            }
        }
    }
    pub fn run(&mut self, duration: f32, forward: bool) {
        *self = match (self.clone(), forward) {
            (no_change @ Self::Start, false) | (no_change @ Self::End, true) => no_change,
            (Self::Start, true) | (Self::End, false) => Self::Between {
                time: if forward { 1.0 } else { 0.0 },
                duration,
                forward,
            },
            (Self::Between { time, .. }, _) => Self::Between {
                time,
                duration,
                forward,
            },
        }
    }
    pub fn get(&self) -> f32 {
        match self {
            Self::Start => 1.0,
            Self::End => 0.0,
            Self::Between { time, .. } => *time,
        }
    }
}

pub struct Tween {
    pub target: f32,
    pub value: f32,
    pub half_life: f32,
}

impl Tween {
    pub fn new(value: f32, half_life: f32) -> Self {
        Self {
            target: value,
            value,
            half_life,
        }
    }
    pub fn tick(&mut self, delta_time: f32) {
        self.value += (self.target - self.value) * (delta_time / self.half_life) * 0.5;
    }
    pub fn set(&mut self, value: f32) {
        self.target = value;
    }
    pub fn get(&self) -> f32 {
        self.value
    }
}
