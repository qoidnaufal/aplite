use util::{Size, Vector2};
use crate::properties::{Alignment, Orientation, Padding};

#[derive(Default, Debug, Clone)]
pub struct Layout {
    next_pos: Vector2<u32>,
    orientation: Orientation,
    alignment: Alignment,
    spacing: u32,
    padding: Padding,
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

    pub(crate) fn set_next_pos<F: FnOnce(&mut Vector2<u32>)>(&mut self, f: F) {
        f(&mut self.next_pos);
    }

    pub(crate) fn adjust_next_pos(&mut self, pos: Vector2<u32>, size: Size<u32>) {
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

    pub(crate) fn reset_to_parent(&mut self, pos: Vector2<u32>, size: Size<u32>) {
        let spacing = self.spacing;
        let half = size / 2;

        // parent orientation
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|p| {
                    p.x = pos.x - half.width;
                    p.y = pos.y + half.height + spacing;
                });
            }
            Orientation::Horizontal => {
                self.set_next_pos(|pos| {
                    pos.y = pos.y - half.height;
                    pos.x = pos.x + half.width + spacing;
                });
            }
        }
    }
}
