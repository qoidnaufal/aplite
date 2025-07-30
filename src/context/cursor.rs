use aplite_types::Vec2f;

use crate::view::ViewId;

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
    pub pos: Vec2f,
    pub offset: Vec2f,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseHover {
    pub pos: Vec2f,
    pub curr: Option<ViewId>,
    pub prev: Option<ViewId>,
    pub z_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub hover: MouseHover,
    pub state: MouseState,
    pub click: MouseClick,
    pub timer: std::time::Duration,
    pub is_dragging: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            hover: MouseHover {
                pos: Vec2f::default(),
                curr: None,
                prev: None,
                z_index: 0,
            },
            state: MouseState {
                action: MouseAction::Released,
                button: MouseButton::Left,
            },
            click: MouseClick {
                pos: Vec2f::default(),
                offset: Vec2f::default(),
            },
            timer: std::time::Duration::from_millis(0),
            is_dragging: false,
        }
    }
}

impl Cursor {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn set_state(&mut self, action: MouseAction, button: MouseButton) {
        self.state = MouseState { action, button };
    }

    pub(crate) fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
        self.set_state(action, button);

        let start = std::time::Instant::now();
        match (self.state.action, self.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.click.pos = self.hover.pos;
            },
            (MouseAction::Released, MouseButton::Left) => {
                self.timer = start.elapsed();
            },
            _ => {}
        }
    }

    pub(crate) fn is_dragging(&self, hover_id: &ViewId) -> bool {
        self.is_clicking()
            && self.hover.curr.is_some_and(|id| &id == hover_id)
            && self.hover.pos != self.click.pos
    }

    pub(crate) fn is_hovering_same_obj(&self) -> bool {
        self.hover.curr == self.hover.prev && self.hover.curr.is_some()
    }

    pub(crate) fn is_idling(&self) -> bool {
        self.is_hovering_same_obj() && !self.is_clicking()
        || self.hover.curr.is_none() && self.hover.prev.is_none()
    }

    pub(crate) fn is_clicking(&self) -> bool {
        matches!(self.state.action, MouseAction::Pressed)
    }
}
