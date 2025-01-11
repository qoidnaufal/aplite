use std::{fs::File, io::{BufReader, Read}, path::{Path, PathBuf}};
use image::GenericImageView;
use math::Size;
use crate::shapes::{Shape, ShapeType};
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
        let texture = image_reader(&self.src);
        eprintln!("{}", texture.data.len());
        eprintln!("{:?}", texture.dimension);

        // Shape::new(Vector2::new(), Size::new(500, 500), Rgb::RED, FilledShape::FilledRectangle)
        Shape::textured(texture.dimension, &texture.data, ShapeType::TexturedRectangle)
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
