use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::{Renderer, DrawArgs, Shape};
use aplite_storage::{IntoComponent, Tree, Table};
use aplite_types::{
    Matrix3x2,
    Rect,
    // Size,
    CornerRadius,
    Paint,
    Rgba,
    Unit,
};

use crate::widget::WidgetId;

#[derive(Clone)]
pub struct WidgetState {
    pub(crate) rect: Rect,
    pub(crate) width: Unit,
    pub(crate) height: Unit,
    pub(crate) transform: Matrix3x2,

    pub(crate) shape: Shape,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) flag: Flag,

    pub(crate) background: Background,

    pub(crate) border: Border,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            width: Unit::default(),
            height: Unit::default(),
            transform: Matrix3x2::identity(),

            shape: Shape::Rect,
            corner_radius: CornerRadius::splat(0),
            flag: Flag::default(),

            background: Background {
                paint: Rgba::WHITE.into(),
                aspect_ratio: AspectRatio::Undefined,
            },

            border: Border {
                paint: Rgba::WHITE.into(),
                width: 0.0,
            },
        }
    }
}

impl IntoComponent for WidgetState {
    type Item = (Rect, (Unit, Unit), Matrix3x2, Shape, CornerRadius, Flag, Background, Border);

    fn into_component(self) -> Self::Item {
        let Self {
            rect,
            width,
            height,
            transform,
            shape,
            corner_radius,
            flag,
            background,
            border,
        } = self;

        (
            rect,
            (width, height),
            transform,
            shape,
            corner_radius,
            flag,
            background,
            border,
        )
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(width: f32, height: f32) -> Self {
        Self {
            width: Unit::Fixed(width),
            height: Unit::Fixed(height),
            background: Background {
                paint: Rgba::TRANSPARENT.into(),
                aspect_ratio: AspectRatio::Undefined,
            },
            border: Border {
                paint: Rgba::TRANSPARENT.into(),
                width: 0.0,
            },
            ..Default::default()
        }
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, width: impl Into<Unit>, height: impl Into<Unit>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            ..self
        }
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.background.paint = paint.into();
        self
    }

    pub fn with_aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.background.aspect_ratio = aspect_ratio;
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.border.paint = paint.into();
        self
    }

    pub fn with_border_width(mut self, val: f32) -> Self {
        self.border.width = val;
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        Self {
            shape,
            ..self
        }
    }

    pub fn with_rotation_deg(mut self, deg: f32) -> Self {
        self.transform.set_rotate_deg(deg);
        self
    }

    pub fn with_rotation_rad(mut self, rad: f32) -> Self {
        self.transform.set_rotate_rad(rad);
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        Self {
            corner_radius: val,
            ..self
        }
    }

    pub fn hoverable(mut self) -> Self {
        self.flag.hoverable = true;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined(u8, u8),
    Source,
    Undefined,
}

#[derive(Clone)]
pub struct Background {
    pub(crate) paint: Paint,
    pub(crate) aspect_ratio: AspectRatio,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag {
    pub(crate) visible: bool,
    pub(crate) focusable: bool,
    pub(crate) hoverable: bool,
    pub(crate) movable: bool,
    pub(crate) needs_redraw: bool,
}

impl Default for Flag {
    fn default() -> Self {
        Self {
            visible: true,
            focusable: false,
            hoverable: false,
            movable: false,
            needs_redraw: true,
        }
    }
}

#[derive(Clone)]
pub struct Border {
    pub(crate) paint: Paint,
    pub(crate) width: f32,
}

pub(crate) struct State {
    pub(crate) common: Table<WidgetId>,
    pub(crate) border: HashMap<WidgetId, Border>,
}

impl Default for State {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl State {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            common: Table::with_capacity(capacity),
            border: HashMap::default(),
        }
    }

    pub(crate) fn insert_default_state(&mut self, id: &WidgetId) {
        let default_state = WidgetState::default();
        self.common.register_component(id, default_state);
    }

    pub(crate) fn insert_state(&mut self, id: &WidgetId, state: WidgetState) {
        self.common.register_component(id, state);
    }

    pub(crate) fn insert_children(&mut self, id: &WidgetId, children: Vec<WidgetId>) {
        self.common.register_component(id, (children,));
    }

    pub(crate) fn render(&self, renderer: &mut Renderer, tree: &Tree<WidgetId>) {
        let mut scene = renderer.scene();
        self.common
            .query::<(&Rect, &Matrix3x2, &Background, &Shape, &CornerRadius, &Flag)>()
            .iter()
            .zip(self.common.entities())
            .for_each(|((rect, transform, background, shape, corner_radius, flag), id)| {
                if flag.visible {
                    let border_state = self.border
                        .get(id)
                        .map(|border| (border.paint.as_paint_ref(), border.width))
                        .unwrap_or((background.paint.as_paint_ref(), 0.0));

                    let draw_args = DrawArgs {
                        rect,
                        transform,
                        background_paint: &background.paint.as_paint_ref(),
                        border_paint: &border_state.0,
                        border_width: &border_state.1,
                        shape,
                        corner_radius,
                    };
                    scene.draw(draw_args);
                }
            });
    }
}
