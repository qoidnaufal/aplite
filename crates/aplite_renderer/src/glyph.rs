use std::sync::Arc;

use fontdue::layout::{Layout, LayoutSettings, CoordinateSystem, TextStyle};
use fontdue::{Font, FontSettings};
use aplite_types::{Rect, Size};
use rustc_hash::FxHashMap;

use crate::atlas::{Atlas, Uv, TextureRef};

const DEFAULT_FONT: &[u8] = include_bytes!("../../../resources/JetBrainsMonoNerdFont-Regular.ttf");

#[derive(Debug, Clone, Copy)]
struct Char {
    c: char,
    s: f32,
}

impl PartialEq for Char {
    fn eq(&self, other: &Self) -> bool {
        self.c == other.c
        && self.s == other.s
    }
}

impl Eq for Char {}

impl std::hash::Hash for Char {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let size = std::mem::size_of_val(self);
            let ptr = self as *const Self as *const u8;
            let slice = std::slice::from_raw_parts(ptr, size);
            state.write(slice);
        }
    }
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
        let ptr_eq = Arc::ptr_eq(&self.bytes, &other.bytes);
        let slice_eq = self.bytes.as_ref() == other.bytes.as_ref();
        ptr_eq && slice_eq
    }
}

impl Eq for GlyphData {}

pub(crate) struct FontHandler {
    font: Font,
    layout: Layout,
    glyphs: FxHashMap<Char, GlyphData>,
    pub(crate) atlas: Atlas,
}

impl FontHandler {
    pub(crate) fn new(device: &wgpu::Device, size: Size) -> Self {
        let settings = FontSettings {
            collection_index: 0,
            scale: 100.,
            load_substitutions: true,
        };

        let font = Font::from_bytes(DEFAULT_FONT, settings).unwrap();
        let layout = Layout::new(CoordinateSystem::PositiveYDown);
        let glyphs = FxHashMap::default();
        let atlas = Atlas::new(device, size, "glyph");

        Self {
            font,
            layout,
            glyphs,
            atlas,
        }
    }

    pub(crate) fn view(&self) -> wgpu::TextureView {
        self.atlas.view()
    }

    pub(crate) fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.atlas.update(device, encoder);
    }

    pub(crate) fn setup(&mut self, text: &str, size: f32, scale: f32, rect: &Rect) {
        self.layout.reset(&LayoutSettings {
            x: rect.x * scale,
            y: rect.y * scale,
            max_width: Some(text.len() as f32 * size * scale),
            ..Default::default()
        });

        let px = size * scale;

        self.layout.append(&[&self.font], &TextStyle::new(text, px, 0));
    }

    pub(crate) fn rasterize_text(
        &mut self,
        text: &str,
        font_size: f32,
        scale: f32,
        rect: &Rect,
    ) -> Vec<(Uv, [f32; 4])> {
        self.setup(text, font_size, scale, rect);

        let mut prims = vec![];
        let s = font_size * scale;
        let b = text.as_bytes();

        for (c, glyph) in b.iter().zip(self.layout.glyphs()) {
            let c = *c as char;
            let uv = {
                let char_data = Char { c, s };

                match self.glyphs.get(&char_data) {
                    Some(glyph_data) => {
                        self.atlas.append(&TextureRef::new(
                            glyph_data.width,
                            glyph_data.height,
                            Arc::downgrade(&glyph_data.bytes))
                        )
                        .unwrap()
                    },
                    None => {
                        let (metric, data) = self.font.rasterize(c, s);
                        let width = metric.width as u32;
                        let height = metric.height as u32;
                        let glyph_data = GlyphData::new(data, width, height);
                        let uv = self.atlas.append(
                            &TextureRef {
                                width,
                                height,
                                bytes: Arc::downgrade(&glyph_data.bytes),
                            }
                        )
                        .unwrap();

                        self.glyphs.insert(char_data, glyph_data);

                        uv
                    },
                }
            };

            let w = (glyph.width as f32 / scale) * 1.6;
            let h = glyph.height as f32 / scale;
            let x = rect.x.midpoint(glyph.x / scale);
            let y = if !c.is_ascii_alphanumeric() {
                rect.y + rect.height / 2. + h
            } else {
                rect.max_y() - h
            };

            prims.push((uv, [x, y, w, h]));
        }

        prims
    }
}
