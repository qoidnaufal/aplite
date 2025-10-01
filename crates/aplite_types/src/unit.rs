#[derive(Clone, Copy)]
pub enum Unit {
    Fixed(f32),
    Grow,
    Fit,
}

impl Default for Unit {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Unit {
    pub fn get(&self) -> f32 {
        match self {
            Unit::Fixed(val) => *val,
            Unit::Grow => 0.0,
            Unit::Fit => 0.0,
        }
    }

    pub fn is_grow(&self) -> bool {
        matches!(self, Self::Grow)
    }

    pub fn is_fit(&self) -> bool {
        matches!(self, Self::Fit)
    }
}

impl From<f32> for Unit {
    fn from(value: f32) -> Self {
        Self::Fixed(value)
    }
}

impl From<u32> for Unit {
    fn from(value: u32) -> Self {
        Self::Fixed(value as f32)
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Unit::Fixed(a), Unit::Fixed(b)) => a == b,
            (Unit::Grow, Unit::Grow) | (Unit::Fit, Unit::Fit) => true,
            _ => false
        }
    }
}

impl Eq for Unit {}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
