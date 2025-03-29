use std::collections::HashMap;

use util::{Size, Vector2};
use crate::{element::Attributes, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Pressed,
    Released,
}

impl From<winit::event::ElementState> for MouseAction {
    fn from(value: winit::event::ElementState) -> Self {
        match value {
            winit::event::ElementState::Pressed => Self::Pressed,
            winit::event::ElementState::Released => Self::Released,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Back => Self::Back,
            winit::event::MouseButton::Forward => Self::Forward,
            winit::event::MouseButton::Other(n) => Self::Other(n),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub action: MouseAction,
    pub button: MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseClick {
    pub pos: Vector2<f32>,
    pub delta: Vector2<f32>,
    pub obj: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseHover {
    pub pos: Vector2<f32>,
    pub curr: Option<NodeId>,
    pub prev: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub hover: MouseHover,
    pub state: MouseState,
    pub click: MouseClick,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            hover: MouseHover {
                pos: Vector2::default(),
                curr: None,
                prev: None,
            },
            state: MouseState {
                action: MouseAction::Released,
                button: MouseButton::Left,
            },
            click: MouseClick {
                pos: Vector2::default(),
                delta: Vector2::default(),
                obj: None,
            },
        }
    }
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_state(&mut self, action: MouseAction, button: MouseButton) {
        self.state = MouseState { action, button };
    }

    pub fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
        self.set_state(action, button);

        match (self.state.action, self.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.click.obj = self.hover.curr;
                self.click.pos = self.hover.pos;
            },
            (MouseAction::Released, MouseButton::Left) => self.click.obj = None,
            _ => {}
        }
    }

    pub fn is_dragging(&self, hover_id: NodeId) -> bool {
        self.click.obj.is_some_and(|click_id| click_id == hover_id)
            && self.hover.pos != self.click.pos
    }

    pub fn is_hovering_same_obj(&self) -> bool {
        self.hover.curr == self.hover.prev
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCtx {
    next_pos: Vector2<u32>,
    alignment_storage: HashMap<NodeId, Alignment>,
    alignment: Alignment,
    spacing: u32,
    padding: u32,
}

impl Default for LayoutCtx {
    fn default() -> Self {
        Self {
            next_pos: Vector2::new(0, 0),
            alignment_storage: HashMap::new(),
            alignment: Alignment::Vertical,
            spacing: 0,
            padding: 0,
        }
    }
}

impl LayoutCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_alignment(&mut self, node_id: NodeId, alignment: Alignment) {
        self.alignment_storage.insert(node_id, alignment);
    }

    fn is_aligned_vertically(&self) -> bool {
        matches!(self.alignment, Alignment::Vertical)
    }

    pub fn align_vertically(&mut self) {
        self.alignment = Alignment::Vertical;
    }

    pub fn align_horizontally(&mut self) {
        self.alignment = Alignment::Horizontal;
    }

    pub fn set_next_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_pos);
    }

    pub fn set_spacing(&mut self, spacing: u32) { self.spacing = spacing }

    pub fn set_padding(&mut self, padding: u32) { self.padding = padding }

    pub fn assign_position(&mut self, attribs: &mut Attributes) {
        let half = attribs.dims / 2;
        attribs.pos = self.next_pos + half;

        let is_aligned_vertically = self.is_aligned_vertically();
        let spacing = self.spacing;
        self.set_next_pos(|next_pos| {
            if is_aligned_vertically {
                next_pos.y = attribs.pos.y + half.height + spacing;
            } else {
                next_pos.x = attribs.pos.x + half.width + spacing;
            }
        });
    }

    pub fn reset_to_parent(
        &mut self,
        parent_id: NodeId,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        let parent_alignment = self.alignment_storage[&parent_id];
        self.alignment = parent_alignment;
        let is_aligned_vertically = self.is_aligned_vertically();
        let padding = self.padding;
        self.set_next_pos(|pos| {
            if is_aligned_vertically {
                pos.x = current_pos.x - half.width;
                pos.y = current_pos.y + half.height + padding;
            } else {
                pos.y = current_pos.y - half.height;
                pos.x = current_pos.x + half.width + padding;
            }
        });
    }
}
