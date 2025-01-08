use std::{fs::File, io::{BufReader, Read}, path::Path};

use image::{GenericImageView, ImageBuffer};

use crate::{
    color::Rgb,
    shapes::{Shape, FilledShape},
    types::{Size, Vector2}
};

use super::{NodeId, Widget, CALLBACKS};

fn image_reader<P: AsRef<Path>>(path: P) -> TextureData {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    let len = reader.read_to_end(&mut buf).unwrap();

    let image = image::load_from_memory(&buf[..len]).unwrap();

    TextureData {
        dimension: image.dimensions().into(),
        data: image.to_rgba8().to_vec(),
    }
}

#[derive(Debug, Clone)]
pub struct TextureData {
    dimension: Size<u32>,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct Image {
    id: NodeId,
}

impl Image {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let texture_data = image_reader(path);
        eprintln!("{}", texture_data.data.len());
        eprintln!("{:?}", texture_data.dimension);
        let id = NodeId::new();

        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, FilledShape::FilledRectangle)
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
