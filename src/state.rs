use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, RefMut};

use aplite_renderer::Shape;
use aplite_reactive::*;
use aplite_types::{
    Matrix3x2,
    Rect,
    Size,
    CornerRadius,
    Paint,
    Rgba,
};

use crate::layout::{AlignV, AlignH, Orientation, Padding};
use crate::context::{Event, PENDING_EVENT};

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined((u32, u32)),
    Source,
    Undefined,
}

#[derive(Clone)]
pub struct WidgetState {
    pub(crate) name: &'static str,
    pub(crate) rect: Rect,
    pub(crate) rotation: f32, // in radians
    // pub(crate) transform: Matrix3x2,
    pub(crate) min_width: Option<f32>,
    pub(crate) min_height: Option<f32>,
    pub(crate) max_width: Option<f32>,
    pub(crate) max_height: Option<f32>,
    pub(crate) align_v: AlignV,
    pub(crate) align_h: AlignH,
    pub(crate) orientation: Orientation,
    pub(crate) padding: Padding,
    pub(crate) spacing: f32,
    // pub(crate) z_index: u32,
    pub(crate) image_aspect_ratio: AspectRatio,
    pub(crate) shape: Shape,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) border_width: f32,
    pub(crate) background_paint: Paint,
    pub(crate) border_paint: Paint,
    pub(crate) dragable: bool,
    pub(crate) hoverable: bool,
    pub(crate) hide: bool,
}

impl std::fmt::Debug for WidgetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            name: "",
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            rotation: 0.0,
            // transform: Matrix3x2::identity(),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            align_v: AlignV::Top,
            align_h: AlignH::Left,
            orientation: Orientation::Vertical,
            spacing: 0.,
            // z_index: 0,
            padding: Padding::default(),
            image_aspect_ratio: AspectRatio::Undefined,
            dragable: false,
            hoverable: false,
            shape: Shape::Rect,
            corner_radius: CornerRadius::splat(0.0),
            background_paint: Paint::Color(Rgba::RED),
            border_paint: Paint::Color(Rgba::WHITE),
            border_width: 0.0,
            hide: false,
        }
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(rect: Rect) -> Self {
        Self {
            name: "Root",
            rect,
            background_paint: Paint::Color(Rgba::TRANSPARENT),
            border_paint: Paint::Color(Rgba::TRANSPARENT),
            ..Default::default()
        }
    }

    pub(crate) fn get_transform(&self, screen: Size) -> Matrix3x2 {
        let rect = self.rect;
        let tx = rect.center_x() / screen.width * 2.0 - 1.0;
        let ty = 1.0 - rect.center_y() / screen.height * 2.0;
        let sx = rect.width / screen.width;
        let sy = rect.height / screen.height;

        Matrix3x2::from_scale_translate(sx, sy, tx, ty)
    }
}

#[derive(Clone, Debug)]
pub struct ViewNode(pub(crate) Rc<RefCell<WidgetState>>);

impl ViewNode {
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(WidgetState::default()));

        Self(state)
    }

    pub(crate) fn window(rect: Rect) -> Self {
        let state = Rc::new(RefCell::new(WidgetState::window(rect)));

        Self(state)
    }

    pub fn node_ref(&self) -> ViewNodeRef {
        let pending_event = PENDING_EVENT.get().unwrap();
        ViewNodeRef(Rc::downgrade(&self.0), *pending_event)
    }

    #[inline(always)]
    pub(crate) fn borrow(&self) -> Ref<'_, WidgetState> {
        self.0.borrow()
    }

    #[inline(always)]
    pub(crate) fn borrow_mut(&self) -> RefMut<'_, WidgetState> {
        self.0.borrow_mut()
    }

    pub fn with_name(self, name: &'static str) -> Self {
        self.0.borrow_mut().name = name;
        self
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, size: impl Into<Size>) -> Self {
        self.0.borrow_mut().rect.set_size(size.into());
        self
    }

    pub fn with_min_width(self, val: f32) -> Self {
        self.0.borrow_mut().min_width = Some(val);
        self
    }

    pub fn with_max_width(self, val: f32) -> Self {
        self.0.borrow_mut().max_width = Some(val);
        self
    }

    pub fn with_min_height(self, val: f32) -> Self {
        self.0.borrow_mut().min_height = Some(val);
        self
    }

    pub fn with_max_height(self, val: f32) -> Self {
        self.0.borrow_mut().max_height = Some(val);
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        self.0.borrow_mut().background_paint = paint.into();
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, color: impl Into<Paint>) -> Self {
        self.0.borrow_mut().border_paint = color.into();
        self
    }

    pub fn with_stroke_width(self, val: f32) -> Self {
        self.0.borrow_mut().border_width = val;
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        self.0.borrow_mut().shape = shape;
        self
    }

    pub fn with_rotation_deg(self, deg: f32) -> Self {
        self.with_rotation_rad(deg.to_radians())
    }

    pub fn with_rotation_rad(self, rad: f32) -> Self {
        self.0.borrow_mut().rotation = rad;
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        self.0.borrow_mut().corner_radius = val;
        self
    }

    pub fn with_horizontal_align(self, align_h: AlignH) -> Self {
        self.0.borrow_mut().align_h = align_h;
        self
    }

    pub fn with_vertical_align(self, align_v: AlignV) -> Self {
        self.0.borrow_mut().align_v = align_v;
        self
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        self.0.borrow_mut().orientation = orientation;
        self
    }

    pub fn hoverable(self) -> Self {
        self.0.borrow_mut().hoverable = true;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ViewNodeRef(Weak<RefCell<WidgetState>>, SignalWrite<Vec<Event>>);

impl ViewNodeRef {
    pub(crate) fn upgrade(&self) -> Option<ViewNode> {
        self.0.upgrade().map(|rc| ViewNode(rc))
    }

    pub fn set_color(&self, color: Rgba<u8>) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().background_paint = color.into();
            self.1.update_untracked(|vec| vec.push(Event::Paint));
        }
    }

    pub fn set_shape(&self, shape: Shape) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().shape = shape;
            self.1.update_untracked(|vec| vec.push(Event::Render));
        }
    }

    pub fn set_rotation_deg(&self, deg: f32) {
        self.set_rotation_rad(deg.to_radians());
    }

    pub fn set_rotation_rad(&self, rad: f32) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().rotation = rad;
            self.1.update_untracked(|vec| vec.push(Event::Render));
        }
    }

    pub fn set_spacing(&self, val: f32) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().spacing = val;
            self.1.update_untracked(|vec| vec.push(Event::Render));
        }
    }

    pub fn hide(&self, val: bool) {
        if let Some(node) = self.upgrade() {
            let prev = node.0.borrow().hide;
            node.0.borrow_mut().hide = val;
            if prev != val {
                self.1.update_untracked(|vec| vec.push(Event::Layout));
            }
        }
    }
}
