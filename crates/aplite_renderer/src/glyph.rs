use std::sync::{Arc, Weak};

use fontdue::layout::{Layout, LayoutSettings, CoordinateSystem, TextStyle};
use fontdue::{Font, FontSettings, Metrics};
use aplite_types::{Rect, Size};
use rustc_hash::FxHashMap;

use crate::atlas::{Atlas, Uv, TextureRef};
use crate::element::Element;

const DEFAULT_FONT: &[u8] = include_bytes!("../../../resources/JetBrainsMonoNerdFont-Regular.ttf");

#[derive(Clone, Copy)]
pub(crate) struct GlyphInfo {
    rect: Option<Rect>,
    metric: Metrics,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Char {
    c: char,
    s: u32,
}

#[derive(Clone)]
struct GlyphData {
    width: u32,
    height: u32,
    bytes: Arc<[u8]>
}

impl GlyphData {
    fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: Arc::from(data.as_slice())
        }
    }

    fn downgrade(&self) -> GlyphRef {
        GlyphRef {
            width: self.width,
            height: self.height,
            bytes: Arc::downgrade(&self.bytes),
        }
    }
}

impl std::hash::Hash for GlyphData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.bytes).addr());
    }
}

impl PartialEq for GlyphData {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bytes, &other.bytes)
    }
}

impl Eq for GlyphData {}

#[derive(Clone)]
struct GlyphRef {
    width: u32,
    height: u32,
    bytes: Weak<[u8]>
}

impl GlyphRef {
    fn upgrade(&self) -> Option<GlyphData> {
        self.bytes.upgrade()
            .map(|bytes| GlyphData {
                width: self.width,
                height: self.height,
                bytes,
            })
    }
}

impl PartialEq for GlyphRef {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.bytes, &other.bytes)
    }
}

impl Eq for GlyphRef {}

impl std::hash::Hash for GlyphRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Weak::as_ptr(&self.bytes).addr());
    }
}

pub(crate) struct FontHandler {
    font: Font,
    layout: Layout,
    glyphs: FxHashMap<Char, GlyphData>,
}

impl FontHandler {
    pub(crate) fn new() -> Self {
        let mut settings = FontSettings::default();
        settings.collection_index = 0;
        settings.scale = 100.;

        let font = Font::from_bytes(DEFAULT_FONT, settings).unwrap();
        let layout = Layout::new(CoordinateSystem::PositiveYUp);
        let glyphs = FxHashMap::default();

        Self {
            font,
            layout,
            glyphs,
        }
    }

    // pub(crate) fn get_glyph(&mut self, c: char, size: f32) -> GlyphRef {
    //     let factor = 65536.0;
    //     let s = (size * factor) as u32;
    //     let hash = Char { c, s };

    //     match self.glyphs.get(&hash) {
    //         Some(data) => data.downgrade(),
    //         None => {
    //             let (metric, data) = self.font.rasterize(c, s as f32 / factor);

    //             let width = metric.width as u32;
    //             let height = metric.height as u32;
    //             let rect = self.allocator.alloc(Size::new(width as f32, height as f32));

    //             let glyph_data = GlyphData::new(data, width, height);
    //             let glyph_ref = glyph_data.downgrade();
    //             self.glyphs.insert(hash, glyph_data);

    //             if let Some(rect) = rect {
    //                 let uv = self.allocator.get_uv(rect);
    //                 self.uvs.insert(glyph_ref.clone(), uv);
    //             }

    //             glyph_ref
    //         },
    //     }
    // }

    pub(crate) fn setup(&mut self, text: &str, size: u32, scale: f32, max_width: Option<f32>) {
        self.layout.reset(&LayoutSettings {
            max_width: max_width.map(|w| w * scale),
            ..Default::default()
        });

        let px = size as f32 * scale;

        self.layout.append(&[&self.font], &TextStyle::new(text, px, 0));
    }

    pub(crate) fn render_text(
        &mut self,
        text: &str,
        size: u32,
        scale: f32,
        max_width: Option<f32>,
        color: &aplite_types::Color,
        atlas: &mut Atlas,
    ) -> Vec<(Element, Uv)> {
        self.setup(text, size, scale, max_width);

        let s = size as f32 * scale;

        let mut prims = vec![];

        for (i, glyph) in self.layout.glyphs().iter().enumerate() {
            let c = text.chars().nth(i).unwrap();
            let uv = {
                let factor = 65536.0;
                let s = (s * factor) as u32;
                let hash = Char { c, s };

                match self.glyphs.get(&hash) {
                    Some(data) => {
                        atlas.append(&TextureRef::new(
                            data.width,
                            data.height,
                            Arc::downgrade(&data.bytes))
                        )
                        .unwrap()
                    },
                    None => {
                        let (metric, data) = self.font.rasterize(c, s as f32 / factor);
                        let width = metric.width as u32;
                        let height = metric.height as u32;

                        let glyph_data = GlyphData::new(data, width, height);
                        let glyph_ref = glyph_data.downgrade();

                        self.glyphs.insert(hash, glyph_data);

                        atlas.append(
                            &TextureRef::new(
                                width,
                                height,
                                glyph_ref.bytes.clone()
                            )
                        )
                        .unwrap()
                    },
                }
            };

            let size = Size::new(glyph.width as f32 / scale, glyph.height as f32 / scale);
            let mut element = Element::new(size).with_shape(crate::element::Shape::Text);

            let packed_color = color.pack_u32();
            element.background = packed_color;
            element.border = packed_color;

            prims.push((element, uv));
        }

        prims
    }
}
