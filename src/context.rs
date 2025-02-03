use std::collections::HashMap;
use util::Vector2;
use crate::NodeId;

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

impl Cursor {
    pub fn new() -> Self {
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

#[derive(Clone, PartialEq)]
pub struct LayoutCtx {
    next_pos: Vector2<u32>,
    next_child_pos: Vector2<u32>,
    // coordinate of the center of the shape
    positions: HashMap<NodeId, Vector2<u32>>,
    parent: HashMap<NodeId, NodeId>,
    children: HashMap<NodeId, Vec<NodeId>>,
}

impl std::fmt::Debug for LayoutCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.positions.iter().try_for_each(|(node_id, pos)| {
            writeln!(f, "Pos | {node_id:?} | {pos:?}")
        })?;
        self.parent.iter().try_for_each(|(child_id, parent_id)| {
            writeln!(f, "Parent | {child_id:?} | {parent_id:?}")
        })?;
        self.children.iter().try_for_each(|(parent_id, children)| {
            writeln!(f, "Children | {parent_id:?} | {children:?}")
        })?;
        Ok(())
    }
}

impl LayoutCtx {
    pub fn new() -> Self {
        Self {
            next_pos: Vector2::new(0, 0),
            next_child_pos: Vector2::new(0, 0),
            positions: HashMap::new(),
            parent: HashMap::new(),
            children: HashMap::new(),
        }
    }

    pub fn set_next_child_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_child_pos);
    }

    pub fn next_child_pos(&self) -> Vector2<u32> { self.next_child_pos }

    pub fn get_parent(&self, node_id: &NodeId) -> Option<&NodeId> {
        self.parent.get(node_id)
    }

    pub fn insert_parent(&mut self, node_id: NodeId, parent_id: NodeId) {
        self.parent.insert(node_id, parent_id);
    }

    pub fn insert_children(&mut self, node_id: NodeId, child_id: NodeId) {
        if let Some(children) = self.children.get_mut(&node_id) {
            children.push(child_id);
        } else {
            self.children.insert(node_id, vec![child_id]);
        }
    }

    pub fn insert_pos(&mut self, node_id: NodeId, pos: Vector2<u32>) {
        self.positions.insert(node_id, Vector2::new(pos.x, pos.y));
    }

    pub fn get_mut_position(&mut self, node_id: NodeId) -> Option<&mut Vector2<u32>> {
        self.positions.get_mut(&node_id)
    }

    pub fn get_position(&self, node_id: &NodeId) -> Option<&Vector2<u32>> {
        self.positions.get(node_id)
    }

    pub fn next_pos(&self) -> Vector2<u32> { self.next_pos }

    pub fn set_next_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_pos);
    }
}
