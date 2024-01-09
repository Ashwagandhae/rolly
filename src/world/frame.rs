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
