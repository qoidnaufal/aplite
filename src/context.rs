use std::{cell::RefCell, collections::HashMap};

use math::{Matrix, Size, Vector2, Vector4};

use crate::NodeId;

thread_local! {
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub cursor: Cursor,
    pub window_size: Size<u32>,
    pub layout: LayoutCtx,
}

impl Context {
    fn new() -> Self {
        Self {
            cursor: Cursor::new(),
            window_size: Size::new(0, 0),
            layout: LayoutCtx::new(),
        }
    }

    pub fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
        self.cursor.set_state(action, button);

        match (self.cursor.state.action, self.cursor.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.cursor.click.obj = self.cursor.hover.curr;
                self.cursor.click.pos = self.cursor.hover.pos;
            },
            (MouseAction::Released, MouseButton::Left) => self.cursor.click.obj = None,
            _ => {}
        }
    }
}

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
    fn new() -> Self {
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
                obj: None,
            },
        }
    }

    pub fn set_state(&mut self,
        action: MouseAction,
        button: MouseButton
    ) {
        self.state = MouseState { action, button };
    }

    pub fn is_dragging(&self, hover_id: NodeId) -> bool {
        self.click.obj.is_some_and(|click_id| click_id == hover_id)
            && self.hover.pos != self.click.pos
    }

    pub fn is_hovering_same_obj(&self) -> bool {
        self.hover.curr == self.hover.prev
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCtx {
    used_space: Vector2<u32>,
    positions: HashMap<NodeId, Vector2<u32>>,
    transform: Matrix<Vector4<f32>, 4>,
}

impl LayoutCtx {
    pub fn new() -> Self {
        Self {
            used_space: Vector2::new(0, 0),
            positions: HashMap::new(),
            transform: Matrix::IDENTITIY,
        }
    }

    pub fn insert(&mut self, node_id: NodeId, pos: Vector2<u32>) {
        self.positions.insert(node_id, Vector2::new(pos.x, pos.y));
    }

    pub fn get_position(&self, node_id: &NodeId) -> Option<&Vector2<u32>> {
        self.positions.get(node_id)
    }

    pub fn used_space(&self) -> Vector2<u32> {
        self.used_space
    }

    pub fn set_used_space<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.used_space);
    }
}
