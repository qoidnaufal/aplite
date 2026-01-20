use aplite_types::{
    Rect,
    Vec2f,
    Size,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignH {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignV {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub top: u8,
    pub bottom: u8,
    pub left: u8,
    pub right: u8,
}

impl Axis {
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl Padding {
    pub const fn new(top: u8, bottom: u8, left: u8, right: u8) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub const fn splat(value: u8) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub fn vertical(&self) -> u8 { self.top + self.bottom }

    pub fn horizontal(&self) -> u8 { self.left + self.right }

    pub fn set_all(&mut self, value: u8) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
}

pub enum LayoutResult {
    Fit,
    UnderFlow,
    OverFlow,
}


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing(pub(crate) u8);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRules {
    pub padding: Padding,
    pub axis: Axis,
    pub align_h: AlignH,
    pub align_v: AlignV,
    pub spacing: Spacing,
}

impl LayoutRules {
    fn offset_x(&self, rect: &Rect) -> f32 {
        let pl = self.padding.left as f32;
        let pr = self.padding.right as f32;

        match self.align_h {
            AlignH::Left => rect.x + pl,
            AlignH::Center => {
                rect.x + rect.width / 2. + pl - pr
            }
            AlignH::Right => rect.max_x() - pr
        }
    }

    fn offset_y(&self, rect: &Rect) -> f32 {
        let pt = self.padding.top as f32;
        let pb = self.padding.bottom as f32;

        match self.align_v {
            AlignV::Top => rect.y + pt,
            AlignV::Middle => {
                rect.y + rect.height / 2. + pt - pb
            }
            AlignV::Bottom => rect.max_y() - pb,
        }
    }

    fn start_pos(&self, rect: Rect, child_dimension_along_axis: f32, child_count: f32) -> Vec2f {
        let offset_x = self.offset_x(&rect);
        let offset_y = self.offset_y(&rect);
        let stretch_factor = self.spacing.0 as f32 * (child_count - 1.);
        let stretch = child_dimension_along_axis + stretch_factor;

        match self.axis {
            Axis::Vertical => {
                let y = match self.align_v {
                    AlignV::Top => offset_y,
                    AlignV::Middle => offset_y - stretch / 2.,
                    AlignV::Bottom => offset_y - stretch,
                };
                Vec2f::new(offset_x, y)
            },
            Axis::Horizontal => {
                let x = match self.align_h {
                    AlignH::Left => offset_x,
                    AlignH::Center => offset_x - stretch / 2.,
                    AlignH::Right => offset_x - stretch,
                };
                Vec2f::new(x, offset_y)
            }
        }
    }
}
