use std::ptr::NonNull;

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
    Other,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Back => Self::Back,
            winit::event::MouseButton::Forward => Self::Forward,
            winit::event::MouseButton::Other(_) => Self::Other,
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
}

#[derive(Default, Debug)]
pub struct MouseCapture {
    pub(crate) id: Option<ViewId>,
    pub(crate) callback: Option<NonNull<dyn Fn()>>,
}

#[derive(Debug)]
pub struct Cursor {
    pub(crate) state: MouseState,
    pub(crate) hover: MouseHover,
    pub(crate) click: MouseClick,
    pub(crate) captured: MouseCapture,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            state: Default::default(),
            hover: Default::default(),
            click: Default::default(),
            captured: Default::default(),
        }
    }
}

pub enum EmittedClickEvent {
    NoOp,
    Captured(ViewId),
    TriggerCallback(NonNull<dyn Fn()>),
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
                self.captured.id = self.hover.curr;

                if let Some(captured) = self.captured.id {
                    EmittedClickEvent::Captured(captured)
                } else {
                    EmittedClickEvent::NoOp
                }
            },
            (MouseAction::Released, MouseButton::Left) => {
                if let Some(id) = self.captured.id.take()
                    && self.hover.curr.is_some_and(|hovered| hovered == id)
                    && let Some(callback) = self.captured.callback.take()
                {
                    EmittedClickEvent::TriggerCallback(callback)
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
            && self.captured.id.is_some()
            && self.hover.pos != self.click.pos
    }

    #[inline(always)]
    pub(crate) fn is_left_clicking(&self) -> bool {
        matches!(self.state.action, MouseAction::Pressed)
            && matches!(self.state.button, MouseButton::Left)
    }
}
