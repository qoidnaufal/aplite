use std::sync::Arc;

use fontdue::layout::{Layout, LayoutSettings, CoordinateSystem, TextStyle};
use fontdue::{Font, FontSettings};
use aplite_types::{Size, Rect};
use rustc_hash::FxHashMap;

use crate::atlas::{Atlas, Uv, TextureRef};
use crate::element::Element;

const DEFAULT_FONT: &[u8] = include_bytes!("../../../resources/JetBrainsMonoNerdFont-Regular.ttf");

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

    pub(crate) fn setup(&mut self, text: &str, size: f32, scale: f32, rect: &Rect) {
        self.layout.reset(&LayoutSettings {
            x: rect.x,
            y: rect.y,
            max_width: Some(rect.width * scale),
            max_height: Some(rect.height * scale),
            ..Default::default()
        });

        let px = size * scale;

        self.layout.append(&[&self.font], &TextStyle::new(text, px, 0));
    }

    pub(crate) fn rasterize_text(
        &mut self,
        text: &str,
        size: f32,
        scale: f32,
        rect: &Rect,
        color: &aplite_types::Color,
        atlas: &mut Atlas,
    ) -> Vec<(Element, (f32, f32), Uv)> {
        self.setup(text, size, scale, rect);

        let s = size * scale;

        let mut prims = vec![];

        let b = text.as_bytes();

        for (i, glyph) in self.layout.glyphs().iter().enumerate() {
            let c = b[i] as char;
            let uv = {
                let factor = 65536.0;
                let s = (s * factor) as u32;
                let hash = Char { c, s };

                match self.glyphs.get(&hash) {
                    Some(glyph_data) => {
                        atlas.append(&TextureRef::new(
                            glyph_data.width,
                            glyph_data.height,
                            Arc::downgrade(&glyph_data.bytes))
                        )
                        .unwrap()
                    },
                    None => {
                        let (metric, data) = self.font.rasterize(c, size * scale);
                        let width = metric.width as u32;
                        let height = metric.height as u32;

                        let glyph_data = GlyphData::new(data, width, height);

                        let uv = atlas.append(
                            &TextureRef::new(
                                width,
                                height,
                                Arc::downgrade(&glyph_data.bytes)
                            )
                        )
                        .unwrap();

                        self.glyphs.insert(hash, glyph_data);

                        uv
                    },
                }
            };

            let packed_color = color.pack_u32();
            let size = Size::new(glyph.width as f32 / scale, glyph.height as f32 / scale);

            let mut element = Element::new(size).with_shape(crate::element::Shape::Text);
            element.background = packed_color;
            element.border = packed_color;

            prims.push((element, (glyph.x, glyph.y), uv));
        }

        prims
    }
}
