use aplite_reactive::RwSignal;
use aplite_types::Vector2;

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
    pub pos: Vector2<f32>,
    pub offset: Vector2<f32>,
    pub obj: Option<ViewId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseHover {
    pub pos: Vector2<f32>,
    pub curr: Option<ViewId>,
    pub prev: Option<ViewId>,
    pub z_index: RwSignal<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub scope: Option<ViewId>,
    pub hover: MouseHover,
    pub state: MouseState,
    pub click: MouseClick,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            scope: None,
            hover: MouseHover {
                pos: Vector2::default(),
                curr: None,
                prev: None,
                z_index: RwSignal::new(0),
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
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn set_state(&mut self, action: MouseAction, button: MouseButton) {
        self.state = MouseState { action, button };
    }

    pub(crate) fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
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

    pub(crate) fn is_dragging(&self, hover_id: &ViewId) -> bool {
        self.click.obj.is_some_and(|click_id| &click_id == hover_id)
            && self.hover.pos != self.click.pos
    }

    pub(crate) fn is_hovering_same_obj(&self) -> bool {
        self.hover.curr == self.hover.prev && self.hover.curr.is_some()
    }

    pub(crate) fn is_idling(&self) -> bool {
        self.is_hovering_same_obj() && self.click.obj.is_none()
    }

    pub(crate) fn is_unscoped(&self) -> bool {
        self.hover.curr.is_none() && self.hover.prev.is_none()
    }
}
