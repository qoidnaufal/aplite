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
    pub offset: Vector2<f32>,
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
                offset: Vector2::default(),
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

    pub fn pos(&self) -> &Vector2<f32> {
        &self.hover.pos
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
    spacing_storage: HashMap<NodeId, u32>,
    padding_storage: HashMap<NodeId, u32>,
    alignment: Alignment,
    spacing: u32,
    padding: u32,
}

impl Default for LayoutCtx {
    fn default() -> Self {
        Self {
            next_pos: Vector2::new(0, 0),
            alignment_storage: HashMap::new(),
            spacing_storage: HashMap::new(),
            padding_storage: HashMap::new(),
            alignment: Alignment::Vertical,
            spacing: 0,
            padding: 0,
        }
    }
}

impl LayoutCtx {
    pub fn new() -> Self { Self::default() }

    pub fn insert_alignment(&mut self, node_id: NodeId, alignment: Alignment) {
        self.alignment_storage.insert(node_id, alignment);
    }

    pub fn insert_spacing(&mut self, node_id: NodeId, spacing: u32) {
        self.spacing_storage.insert(node_id, spacing);
    }

    pub fn insert_padding(&mut self, node_id: NodeId, padding: u32) {
        self.padding_storage.insert(node_id, padding);
    }

    pub fn get_spacing(&self) -> u32 { self.spacing }

    pub fn get_padding(&self) -> u32 { self.padding }

    pub fn set_to_parent_alignment(&mut self, parent_id: NodeId) {
        self.alignment = self.alignment_storage[&parent_id];
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

    pub fn set_spacing(&mut self, node_id: &NodeId) {
        self.spacing = self.spacing_storage[node_id];
    }

    pub fn set_padding(&mut self, node_id: &NodeId) {
        self.padding = self.padding_storage[node_id];
    }

    pub fn assign_position(&mut self, attribs: &mut Attributes) {
        let half = attribs.dims / 2;
        attribs.pos = self.next_pos + half;
        let spacing = self.spacing;
        match self.alignment {
            Alignment::Vertical => {
                self.set_next_pos(|p| p.y = attribs.pos.y + half.height + spacing);
            }
            Alignment::Horizontal => {
                self.set_next_pos(|p| p.x = attribs.pos.x + half.width + spacing);
            }
        }
    }

    pub fn reset_to_parent(
        &mut self,
        parent_id: NodeId,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        self.set_to_parent_alignment(parent_id);
        let padding = self.padding;
        match self.alignment {
            Alignment::Vertical => {
                self.set_next_pos(|pos| {
                    pos.x = current_pos.x - half.width;
                    pos.y = current_pos.y + half.height + padding;
                });
            }
            Alignment::Horizontal => {
                self.set_next_pos(|pos| {
                    pos.y = current_pos.y - half.height;
                    pos.x = current_pos.x + half.width + padding;
                });
            }
        }
    }

    pub fn print_alignment(&self, node_id: &NodeId) {
        eprintln!("{node_id:?} | {:?}", self.alignment)
    }
}
