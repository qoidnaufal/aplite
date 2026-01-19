use aplite_types::Vec2f;

use crate::context::ViewId;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Pressed,
    #[default]
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub(crate) action: MouseAction,
    pub(crate) button: MouseButton,
}

#[derive(Default, Debug)]
pub struct MouseHover {
    pub(crate) pos: Vec2f,
    pub(crate) curr: Option<ViewId>,
}

#[derive(Default, Debug)]
pub struct MouseClick {
    pub(crate) pos: Vec2f,
    pub(crate) offset: Vec2f,
    pub(crate) captured: Option<ViewId>,
}

#[derive(Default, Debug)]
pub struct Cursor {
    pub(crate) state: MouseState,
    pub(crate) hover: MouseHover,
    pub(crate) click: MouseClick,
    pub(crate) is_dragging: bool,
}

pub enum EmittedClickEvent {
    NoOp,
    Captured(ViewId),
    TriggerCallback(ViewId),
}

impl Cursor {
    #[inline(always)]
    pub fn hover_pos(&self) -> Vec2f {
        self.hover.pos
    }

    #[inline(always)]
    pub fn click_pos(&self) -> Vec2f {
        self.click.pos
    }

    #[inline(always)]
    pub fn button(&self) -> MouseButton {
        self.state.button
    }

    #[inline(always)]
    pub fn action(&self) -> MouseAction {
        self.state.action
    }

    #[inline(always)]
    fn set_state(&mut self, action: MouseAction, button: MouseButton) {
        self.state = MouseState { action, button };
    }

    pub(crate) fn process_click_event(
        &mut self,
        action: MouseAction,
        button: MouseButton,
    ) -> EmittedClickEvent {
        self.set_state(action, button);

        match (self.state.action, self.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.click.pos = self.hover.pos;
                self.click.captured = self.hover.curr;

                if let Some(captured) = self.click.captured {
                    EmittedClickEvent::Captured(captured)
                } else {
                    EmittedClickEvent::NoOp
                }
            },
            (MouseAction::Released, MouseButton::Left) => {
                let captured = self.click.captured.take();
                let was_dragging = self.is_dragging;
                self.is_dragging = false;

                if let Some(captured) = captured
                    && self.hover.curr.is_some_and(|hovered| hovered == captured)
                    && !was_dragging
                {
                    EmittedClickEvent::TriggerCallback(captured)
                } else {
                    EmittedClickEvent::NoOp
                }
            },
            _ => EmittedClickEvent::NoOp,
        }
    }

    #[inline(always)]
    pub(crate) fn is_dragging(&self) -> bool {
        self.is_left_clicking()
            && self.click.captured.is_some()
            && self.hover.pos != self.click.pos
    }

    #[inline(always)]
    pub(crate) fn is_left_clicking(&self) -> bool {
        matches!(self.state.action, MouseAction::Pressed)
            && matches!(self.state.button, MouseButton::Left)
    }
}
