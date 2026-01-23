use std::sync::Arc;

use aplite_types::{Rect, Color, rgb};
use aplite_renderer::Scene;

use crate::view::IntoView;
use crate::widget::{Widget, Renderable};
use crate::context::{BuildCx, LayoutCx, CursorCx};
use crate::layout::Axis;

pub fn text<T: std::fmt::Display>(text: T) -> Text<T> {
    Text {
        text,
        style_fn: None,
    }
}

pub struct Text<T> {
    text: T,
    style_fn: Option<Box<dyn Fn(&mut TextElement)>>,
}

impl<T: std::fmt::Display + 'static> Widget for Text<T> {
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        let mut text_element = TextElement {
            text: ArcStr::new(self.text.to_string()),
            color: rgb(0x000000),
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut text_element);
        }

        cx.register_element(text_element)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let element = cx.get_element::<TextElement>().unwrap();
        let len = element.text.len() as f32;

        let node = match cx.rules.axis {
            Axis::Horizontal => {
                let width = cx.bound.width.min(len);
                cx.bound.x += width + cx.rules.spacing.0 as f32;
                Rect::new(cx.bound.x, cx.bound.y, width, cx.bound.height)
            },
            Axis::Vertical =>  {
                let height = cx.bound.height.min(len);
                cx.bound.y += height + cx.rules.spacing.0 as f32;
                Rect::new(cx.bound.x, cx.bound.y, cx.bound.width, height)
            },
        };

        cx.set_node(node);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        let rect = cx.get_layout_node().unwrap();
        let hovered = rect.contains(cx.hover_pos());

        if hovered {
            cx.set_id()
        }

        hovered
    }
}

impl<T: std::fmt::Display + 'static> IntoView for Text<T> {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

#[derive(PartialEq, Eq)]
pub struct TextElement {
    text: ArcStr,
    pub color: Color,
}

impl std::fmt::Debug for TextElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextElement")
            .finish_non_exhaustive()
    }
}

impl Renderable for TextElement {
    fn render(&self, _rect: &Rect, _scene: &mut Scene) {}

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn equal(&self, other: &dyn Renderable) -> bool {
        if other.type_id() == self.type_id() {
            unsafe {
                let ptr = other as *const dyn Renderable as *const Self;
                (&*ptr).eq(self)
            }
        } else {
            false
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ArcStr(Arc<str>);

impl ArcStr {
    fn new(text: impl AsRef<str>) -> Self {
        Self(Arc::from(text.as_ref()))
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

macro_rules! impl_widget {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) -> bool {
                let text_element = TextElement {
                    text: self.into(),
                    color: rgb(0x000000),
                };

                cx.register_element(text_element)
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                let element = cx.get_element::<TextElement>().unwrap();
                let len = element.text.len() as f32;

                let node = match cx.rules.axis {
                    Axis::Horizontal => {
                        let width = cx.bound.width.min(len);
                        cx.bound.x += width + cx.rules.spacing.0 as f32;
                        Rect::new(cx.bound.x, cx.bound.y, width, cx.bound.height)
                    },
                    Axis::Vertical =>  {
                        let height = cx.bound.height.min(len);
                        cx.bound.y += height + cx.rules.spacing.0 as f32;
                        Rect::new(cx.bound.x, cx.bound.y, cx.bound.width, height)
                    },
                };

                cx.set_node(node);
            }

            fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
                let rect = cx.get_layout_node().unwrap();
                let hovered = rect.contains(cx.hover_pos());

                if hovered {
                    cx.set_id()
                }

                hovered
            }
        }

        impl IntoView for $name {
            type View = Self;

            fn into_view(self) -> Self::View {
                self
            }
        }

        impl From<&$name> for ArcStr {
            fn from(num: &$name) -> Self {
                ArcStr::new(num.to_string())
            }
        }

        impl From<$name> for ArcStr {
            fn from(num: $name) -> Self {
                ArcStr::new(num.to_string())
            }
        }

        impl TextStyle for $name {}
    };

    ($next:ty, $($rest:ty),*) => {
        impl_widget!{ $next }
        impl_widget!{ $($rest),* }
    };
}

pub trait TextStyle: std::fmt::Display + Sized + 'static {
    fn style<F>(self, style_fn: F) -> Text<Self>
    where
        F: Fn(&mut TextElement) + 'static,
    {
        Text {
            text: self,
            style_fn: Some(Box::new(style_fn)),
        }
    }
}

impl_widget!(
    u8,    i8,
    u16,   i16,
    u32,   i32,
    u64,   i64,
    usize, isize,
    u128,  i128,
    f32,   f64,
    &'static str,
    String
);
