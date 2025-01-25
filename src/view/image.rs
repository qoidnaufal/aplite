use std::path::{Path, PathBuf};
use crate::{callback::CALLBACKS, context::CONTEXT, shapes::{Shape, ShapeKind}};
use super::{AnyView, IntoView, NodeId, View};

pub fn image<P: AsRef<Path>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    src: PathBuf,
}

impl Image {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let id = NodeId::new();
        Self { id, src: path.as_ref().to_path_buf() }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::textured(self.src.clone(), ShapeKind::TexturedRectangle)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Image {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[AnyView]> {
        None
    }

    fn shape(&self) -> Shape {
        self.shape()
    }

    fn layout(&self) {
        let dimensions = self.shape().dimensions;
        CONTEXT.with_borrow_mut(|cx| {
            if cx.layout.get_position(&self.id()).is_none() {
                let used_space = cx.layout.used_space();
                cx.layout.insert(self.id(), (0, used_space.y).into());
                cx.layout.set_used_space(|space| space.y += dimensions.height);
            }
        });
    }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
