// pub struct Width(Length);
// pub struct Height(Length);

#[derive(Clone, Copy)]
pub enum Length {
    Fixed(f32),
    Grow,
    Fit,
}

impl Default for Length {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Length {
    pub fn get(&self, constraint: f32) -> f32 {
        match self {
            Length::Fixed(val) => *val,
            Length::Grow => constraint,
            Length::Fit => constraint,
        }
    }

    pub fn is_grow(&self) -> bool {
        matches!(self, Self::Grow)
    }

    pub fn is_fit(&self) -> bool {
        matches!(self, Self::Fit)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Fixed(value)
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self::Fixed(value as f32)
    }
}

impl PartialEq for Length {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Length::Fixed(a), Length::Fixed(b)) => a == b,
            (Length::Grow, Length::Grow) | (Length::Fit, Length::Fit) => true,
            _ => false
        }
    }
}

impl Eq for Length {}

impl std::fmt::Debug for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
