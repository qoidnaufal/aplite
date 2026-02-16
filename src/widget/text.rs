use std::sync::{Arc, Weak};

use aplite_renderer::Scene;
use aplite_types::{
    Matrix3x2,
    Rect,
    Color,
    rgb
};

use crate::view::IntoView;
use crate::widget::{Widget, Renderable};
use crate::context::{BuildCx, LayoutCx, CursorCx};
use crate::layout::{AlignH, AlignV, Axis};

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
            text: TextData::new(self.text.to_string()),
            size: 25.,
            color: rgb(0x000000),
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut text_element);
        }

        cx.register_element(text_element)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let element = cx.get_element::<TextElement>().unwrap();
        let size = element.size;
        let bound = cx.bound.width.min(cx.bound.height);

        let node = match cx.rules.axis {
            Axis::Horizontal => {
                let bound_size = bound.min(size);

                let x = match cx.rules.align_h {
                    AlignH::Left => cx.bound.x,
                    AlignH::Center => cx.bound.x - bound_size / 2.,
                    AlignH::Right => cx.bound.x - bound_size,
                };

                let y = match cx.rules.align_v {
                    AlignV::Top => cx.bound.y,
                    AlignV::Middle => cx.bound.y - bound_size / 2.,
                    AlignV::Bottom => cx.bound.y - bound_size,
                };

                cx.bound.x += bound_size + cx.rules.spacing.0 as f32;

                Rect::new(x, y, bound_size, bound_size)
            },
            Axis::Vertical =>  {
                let bound_height = bound.min(size);

                let x = match cx.rules.align_h {
                    AlignH::Left => cx.bound.x,
                    AlignH::Center => cx.bound.x - bound_height / 2.,
                    AlignH::Right => cx.bound.x - bound_height,
                };

                let y = match cx.rules.align_v {
                    AlignV::Top => cx.bound.y,
                    AlignV::Middle => cx.bound.y - bound_height / 2.,
                    AlignV::Bottom => cx.bound.y - bound_height,
                };

                cx.bound.y += bound_height + cx.rules.spacing.0 as f32;

                Rect::new(x, y, bound_height, bound_height)
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

pub struct TextElement {
    text: TextData,
    size: f32,
    pub color: Color,
}

impl PartialEq for TextElement {
    fn eq(&self, other: &Self) -> bool {
        self.text.eq(&other.text)
            && self.size.eq(&other.size)
            && self.color.eq(&other.color)
    }
}

impl Eq for TextElement {}

impl std::fmt::Debug for TextElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextElement")
            .finish_non_exhaustive()
    }
}

impl Renderable for TextElement {
    fn render(&self, rect: &Rect, scene: &mut Scene) {
        scene.draw_text(
            &self.text.0,
            &self.size,
            rect,
            &Matrix3x2::identity(),
            &self.color
        );
    }

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn equal(&self, other: &dyn Renderable) -> bool {
        if other.type_id() == self.type_id() {
            unsafe {
                let ptr = other as *const dyn Renderable as *const Self;
                return (&*ptr).eq(self)
            }
        }

        false
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TextData(Arc<str>);

impl TextData {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self(Arc::from(text.as_ref()))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn downgrade(&self) -> TextRef {
        TextRef(Arc::downgrade(&self.0))
    }
}

impl AsRef<str> for TextData {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone)]
pub struct TextRef(Weak<str>);

impl TextRef {
    pub fn upgrade(&self) -> Option<TextData> {
        self.0.upgrade().map(TextData)
    }
}

impl std::hash::Hash for TextRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Weak::as_ptr(&self.0).addr());
    }
}

impl PartialEq for TextRef {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TextRef {}

macro_rules! impl_widget {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) -> bool {
                let text_element = TextElement {
                    text: self.into(),
                    size: 50.,
                    color: rgb(0x000000),
                };

                cx.register_element(text_element)
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                let element = cx.get_element::<TextElement>().unwrap();
                let size = element.size;
                let bound = cx.bound.width.min(cx.bound.height);

                let node = match cx.rules.axis {
                    Axis::Horizontal => {
                        let bound_width = bound.min(size);

                        let x = match cx.rules.align_h {
                            AlignH::Left => cx.bound.x,
                            AlignH::Center => cx.bound.x - bound_width / 2.,
                            AlignH::Right => cx.bound.x - bound_width,
                        };

                        let y = match cx.rules.align_v {
                            AlignV::Top => cx.bound.y,
                            AlignV::Middle => cx.bound.y - bound_width / 2.,
                            AlignV::Bottom => cx.bound.y - bound_width,
                        };

                        cx.bound.x += bound_width + cx.rules.spacing.0 as f32;

                        Rect::new(x, y, bound_width, bound_width)
                    },
                    Axis::Vertical =>  {
                        let bound_height = bound.min(size);

                        let x = match cx.rules.align_h {
                            AlignH::Left => cx.bound.x,
                            AlignH::Center => cx.bound.x - bound_height / 2.,
                            AlignH::Right => cx.bound.x - bound_height,
                        };

                        let y = match cx.rules.align_v {
                            AlignV::Top => cx.bound.y,
                            AlignV::Middle => cx.bound.y - bound_height / 2.,
                            AlignV::Bottom => cx.bound.y - bound_height,
                        };

                        cx.bound.y += bound_height + cx.rules.spacing.0 as f32;

                        Rect::new(x, y, bound_height, bound_height)
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

        impl From<&$name> for TextData {
            fn from(num: &$name) -> Self {
                TextData::new(num.to_string())
            }
        }

        impl From<$name> for TextData {
            fn from(num: $name) -> Self {
                TextData::new(num.to_string())
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
