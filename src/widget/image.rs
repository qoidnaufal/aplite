use std::path::{Path, PathBuf};
use crate::shapes::{Shape, ShapeKind};
use super::{NodeId, Widget, CALLBACKS};

pub fn image<P: AsRef<Path>>(src: P) -> Image {
    Image::new(src)
}

#[derive(Debug)]
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

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl Widget for Image {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl Widget for &Image {
    fn id(&self) -> NodeId {
        (*self).id()
    }

    fn shape(&self) -> Shape {
        (*self).shape()
    }
}
