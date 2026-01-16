// pub struct Width(Length);
// pub struct Height(Length);

#[derive(Clone, Copy)]
pub enum Length {
    Grow,
    Fixed(f32),
    FitContent,
}

impl Default for Length {
    fn default() -> Self {
        Self::Grow
    }
}

impl Length {
    pub fn is_grow(&self) -> bool {
        matches!(self, Self::Grow)
    }

    pub fn is_fit(&self) -> bool {
        matches!(self, Self::FitContent)
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
            (Length::Grow, Length::Grow)
            | (Length::FitContent, Length::FitContent) => true,
            (Length::Fixed(a), Length::Fixed(b)) => a == b,
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
