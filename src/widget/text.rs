use std::borrow::Cow;

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

pub fn text<IV>(text: IV) -> Text<IV>
where
    IV: IntoView,
    IV::View: std::fmt::Display,
{
    Text {
        text: text.into_view(),
        style_fn: None,
    }
}

pub struct Text<IV>
where
    IV: IntoView,
    IV::View: std::fmt::Display,
{
    text: IV::View,
    style_fn: Option<Box<dyn Fn(&mut TextElement)>>,
}

impl<IV> Text<IV>
where
    IV: IntoView,
    IV::View: std::fmt::Display,
{
    pub fn style(self, f: impl Fn(&mut TextElement) + 'static) -> Self {
        Self {
            text: self.text,
            style_fn: Some(Box::new(f)),
        }
    }
}

impl<IV> Widget for Text<IV>
where
    IV: IntoView,
    IV::View: std::fmt::Display,
{
    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        let mut text_element = TextElement {
            text: Cow::from(self.text.to_string()),
            ..Default::default()
        };

        if let Some(style_fn) = self.style_fn.as_ref() {
            style_fn(&mut text_element);
        }

        cx.add_or_update_element(text_element)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let element = cx.get_element::<TextElement>().unwrap();
        let size = element.size;
        let len = element.text.len();
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

                match element.axis {
                    Axis::Horizontal => {
                        cx.bound.x += bound_width * len as f32 + cx.rules.spacing.0 as f32;
                    }
                    Axis::Vertical => {
                        cx.bound.x += bound_width + cx.rules.spacing.0 as f32;
                    }
                }

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

                match element.axis {
                    Axis::Horizontal => {
                        cx.bound.y += bound_height + cx.rules.spacing.0 as f32;
                    }
                    Axis::Vertical => {
                        cx.bound.y += bound_height * len as f32 + cx.rules.spacing.0 as f32;
                    }
                }

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

impl<IV> IntoView for Text<IV>
where
    IV: IntoView,
    IV::View: std::fmt::Display,
{
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

pub struct TextElement {
    text: Cow<'static, str>,
    pub size: f32,
    pub color: Color,
    pub axis: Axis,
    pub align_h: AlignH,
    pub align_v: AlignV,
}

impl Default for TextElement {
    fn default() -> Self {
        Self {
            text: Cow::from(""),
            size: 50.,
            color: rgb(0x000000),
            axis: Axis::Horizontal,
            align_h: AlignH::Left,
            align_v: AlignV::Middle,
        }
    }
}

impl PartialEq for TextElement {
    fn eq(&self, other: &Self) -> bool {
        self.text.as_ref().eq(other.text.as_ref())
            && self.size.eq(&other.size)
            && self.color.eq(&other.color)
    }
}

impl Eq for TextElement {}

impl std::fmt::Debug for TextElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextElement")
            .field("text", &self.text)
            .finish_non_exhaustive()
    }
}

impl Renderable for TextElement {
    fn render(&self, rect: &Rect, scene: &mut Scene) {
        scene.draw_text(
            self.text.as_ref(),
            &self.size,
            rect,
            &Matrix3x2::identity(),
            &self.color
        );
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

macro_rules! impl_display_primitive {
    ($name:ty) => {
        impl Widget for $name {
            fn build(&self, cx: &mut BuildCx<'_>) -> bool {
                let text_element = TextElement {
                    text: Cow::from(self.to_string()),
                    ..Default::default()
                };

                cx.add_or_update_element(text_element)
            }

            fn layout(&self, cx: &mut LayoutCx<'_>) {
                let element = cx.get_element::<TextElement>().unwrap();
                let size = element.size;
                let len = element.text.len();
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

                        match element.axis {
                            Axis::Horizontal => {
                                cx.bound.x += bound_width * len as f32 + cx.rules.spacing.0 as f32;
                            }
                            Axis::Vertical => {
                                cx.bound.x += bound_width + cx.rules.spacing.0 as f32;
                            }
                        }

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

                        match element.axis {
                            Axis::Horizontal => {
                                cx.bound.y += bound_height + cx.rules.spacing.0 as f32;
                            }
                            Axis::Vertical => {
                                cx.bound.y += bound_height * len as f32 + cx.rules.spacing.0 as f32;
                            }
                        }

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

        impl TextStyle for $name {}
    };

    ($next:ty, $($rest:ty),*) => {
        impl_display_primitive!{ $next }
        impl_display_primitive!{ $($rest),* }
    };
}

pub trait TextStyle: IntoView + std::fmt::Display + Sized + 'static {
    fn style<F>(self, style_fn: F) -> Text<Self>
    where
        F: Fn(&mut TextElement) + 'static,
        Self::View: std::fmt::Display,
    {
        Text {
            text: self.into_view(),
            style_fn: Some(Box::new(style_fn)),
        }
    }
}

impl_display_primitive!(
    u8,    i8,
    u16,   i16,
    u32,   i32,
    u64,   i64,
    usize, isize,
    u128,  i128,
    f32,   f64,
    char,
    &'static str,
    String
);

pub trait Integer where Self: Sized
    + std::ops::Add + std::ops::AddAssign
    + std::ops::Sub + std::ops::SubAssign
    + std::ops::Mul + std::ops::MulAssign
    + std::ops::Div + std::ops::DivAssign
    + PartialEq + Eq
    + PartialOrd + Ord
    + Clone + Copy
{
    const MAX_STR_LEN: usize;

    fn as_str_slice(self) -> &'static str;
}

macro_rules! impl_unsigned_integer {
    ($name:ident) => {
        impl Integer for $name {
            const MAX_STR_LEN: usize = Self::MAX.ilog(10) as usize + 1;

            fn as_str_slice(self) -> &'static str {
                let mut buffer = [0u8; Self::MAX_STR_LEN];
                let mut curr = self;
                let mut i = Self::MAX_STR_LEN;

                while curr > 0 {
                    let last_digit = curr % 10;
                    curr /= 10;
                    unsafe { *buffer.get_unchecked_mut(i - 1) = last_digit as u8 + b'0' };
                    i -= 1;
                }

                unsafe {
                    let s = str::from_utf8_unchecked(&buffer[i..]);
                    std::mem::transmute_copy(&s)
                }
            }
        }
    };
    ($first:ident, $($rest:ident),*) => {
        impl_unsigned_integer!($first);
        impl_unsigned_integer!($($rest),*);
    };
}

impl_unsigned_integer!(u8, u16, u32, u64, usize);

macro_rules! impl_signed_integer {
    ($name:ident) => {
        impl Integer for $name {
            const MAX_STR_LEN: usize = Self::MAX.ilog(10) as usize + 2;

            fn as_str_slice(self) -> &'static str {
                let is_negative = self.is_negative();

                let mut buffer = [0u8; Self::MAX_STR_LEN];
                let mut i = Self::MAX_STR_LEN;
                let mut curr = if is_negative {
                    self.unsigned_abs()
                } else {
                    self as _
                };

                while curr > 0 {
                    let last_digit = curr % 10;
                    curr /= 10;
                    unsafe { *buffer.get_unchecked_mut(i - 1) = last_digit as u8 + b'0' };
                    i -= 1;
                }

                if is_negative {
                    buffer[i - 1] = b'-';
                    i -= 1;
                }

                unsafe {
                    let s = str::from_utf8_unchecked(&buffer[i..]);
                    std::mem::transmute_copy(&s)
                }
            }
        }
    };
    ($first:ident, $($rest:ident),*) => {
        impl_signed_integer!($first);
        impl_signed_integer!($($rest),*);
    };
}

impl_signed_integer!(i8, i16, i32, i64, isize);

#[cfg(test)]
mod integer_test {
    use super::*;

    #[test]
    fn integer_to_str() {
        let a = 69_u8.as_str_slice();
        assert_eq!(a, "69");

        let c = (- 69).as_str_slice();
        assert_eq!(c.len(), 3);
        // assert_eq!("-69", c.to_owned());
    }
}
