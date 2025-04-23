use util::{Size, Vector2};
use crate::style::{Alignment, Orientation, Padding};

#[derive(Debug, Clone)]
pub struct Layout {
    next_pos: Vector2<u32>,
    orientation: Orientation,
    alignment: Alignment,
    spacing: u32,
    padding: Padding,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            next_pos: Vector2::default(),
            orientation: Orientation::default(),
            alignment: Alignment::default(),
            padding: Padding::default(),
            spacing: 0,
        }
    }
}

impl Layout {
    pub(crate) fn new() -> Self { Self::default() }

    pub(crate) fn next_pos(&self) -> Vector2<u32> {
        self.next_pos
    }

    pub(crate) fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub(crate) fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub(crate) fn set_alignment(&mut self, align: Alignment) {
        self.alignment = align;
    }

    pub(crate) fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub(crate) fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }

    pub(crate) fn set_padding(&mut self, padding: Padding) {
        self.padding = padding;
    }

    pub(crate) fn set_next_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_pos);
    }

    pub(crate) fn assign_position(&mut self, pos: Vector2<u32>, size: Size<u32>) {
        let spacing = self.spacing;
        let half = size / 2;
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|p| p.y = pos.y + half.height + spacing);
            }
            Orientation::Horizontal => {
                self.set_next_pos(|p| p.x = pos.x + half.width + spacing);
            }
        };
    }

    pub(crate) fn reset_to_parent(
        &mut self,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        let spacing = self.spacing;

        // parent orientation
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|pos| {
                    pos.x = current_pos.x - half.width;
                    pos.y = current_pos.y + half.height + spacing;
                });
            }
            Orientation::Horizontal => {
                self.set_next_pos(|pos| {
                    pos.y = current_pos.y - half.height;
                    pos.x = current_pos.x + half.width + spacing;
                });
            }
        }
    }
}
