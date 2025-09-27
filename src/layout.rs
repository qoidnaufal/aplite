use aplite_types::{Rect, Vec2f, Size, Matrix3x2, Paint, Rgba, CornerRadius};
use aplite_storage::{SparseIndices, Array, ImmutableArray, Tree, Entity};
use aplite_renderer::{Renderer, DrawArgs, Shape};

use crate::state::{WidgetState, AspectRatio, Flag};
use crate::widget::{Widget, WidgetId};

// pub struct LayoutCx {
//     pub(crate) next_pos: Vec2f,
//     pub(crate) rules: Rules,
// }

// impl LayoutCx {
//     pub fn new(parent: &dyn Widget) -> Self {
//         let node = parent.node_ref().unwrap().upgrade();
//         let rules = Rules::new(&node.borrow());

//         let(total_size, len) = parent.children_ref()
//             .map(|children| {
//                 (
//                     children.iter()
//                         .map(|child| {
//                             let rect = child.node_ref().unwrap().upgrade().borrow().rect;
//                             match rules.orientation {
//                                 Orientation::Vertical => rect.size().height,
//                                 Orientation::Horizontal => rect.size().width,
//                             }
//                         })
//                         .sum::<f32>(),
//                     children.len() as f32
//                 )
//             }).unwrap_or_default();

//         let next_pos = rules.start_pos(total_size, len);

//         Self {
//             rules,
//             next_pos,
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) struct Rules {
//     pub(crate) rect: Rect,
//     pub(crate) orientation: Orientation,
//     pub(crate) align_h: AlignH,
//     pub(crate) align_v: AlignV,
//     pub(crate) padding: Padding,
//     pub(crate) spacing: u8,
// }

// impl Rules {
//     pub(crate) fn new(state: &WidgetState) -> Self {
//         Self {
//             rect: state.rect,
//             orientation: state.orientation,
//             align_h: state.align_h,
//             align_v: state.align_v,
//             padding: state.padding,
//             spacing: state.spacing,
//         }
//     }

//     fn offset_x(&self) -> f32 {
//         let pl = self.padding.left as f32;
//         let pr = self.padding.right as f32;

//         match self.align_h {
//             AlignH::Left => self.rect.x + pl,
//             AlignH::Center => {
//                 self.rect.x + self.rect.width / 2. + pl - pr
//             }
//             AlignH::Right => self.rect.max_x() - pr
//         }
//     }

//     fn offset_y(&self) -> f32 {
//         let pt = self.padding.top as f32;
//         let pb = self.padding.bottom as f32;

//         match self.align_v {
//             AlignV::Top => self.rect.y + pt,
//             AlignV::Middle => {
//                 self.rect.y + self.rect.height / 2. + pt - pb
//             }
//             AlignV::Bottom => self.rect.max_y() - pb,
//         }
//     }

//     fn start_pos(&self, child_total_size: f32, len: f32) -> Vec2f {
//         let offset_x = self.offset_x();
//         let offset_y = self.offset_y();
//         let stretch_factor = self.spacing as f32 * (len - 1.);
//         let stretch = child_total_size + stretch_factor;

//         match self.orientation {
//             Orientation::Vertical => {
//                 let y = match self.align_v {
//                     AlignV::Top => offset_y,
//                     AlignV::Middle => offset_y - stretch / 2.,
//                     AlignV::Bottom => offset_y - stretch,
//                 };
//                 Vec2f::new(offset_x, y)
//             },
//             Orientation::Horizontal => {
//                 let x = match self.align_h {
//                     AlignH::Left => offset_x,
//                     AlignH::Center => offset_x - stretch / 2.,
//                     AlignH::Right => offset_x - stretch,
//                 };
//                 Vec2f::new(x, offset_y)
//             }
//         }
//     }
// }

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignH {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignV {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Padding {
    pub top: u8,
    pub bottom: u8,
    pub left: u8,
    pub right: u8,
}

impl Orientation {
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl Padding {
    pub const fn new(top: u8, bottom: u8, left: u8, right: u8) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub const fn splat(value: u8) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub(crate) fn vertical(&self) -> u8 { self.top + self.bottom }

    pub(crate) fn horizontal(&self) -> u8 { self.left + self.right }

    pub fn set_all(&mut self, value: u8) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
}

// impl<T: Widget + Sized + 'static> Layout for T {}

// pub(crate) trait Layout: Widget + Sized + 'static {
//     fn calculate_layout(&self, cx: &mut LayoutCx) {
//         if self.layout(cx) && let Some(children) = self.children_ref() {
//             let mut this_cx = LayoutCx::new(self);
//             children.iter()
//                 .for_each(|child| child.calculate_layout(&mut this_cx));
//         }
//     }

//     fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
//         let node = self.node_ref().unwrap().upgrade();
//         if node.borrow().flag.is_hidden() { return Size::default() }

//         let state = node.borrow();
//         let padding = state.padding;
//         let orientation = state.orientation;
//         let spacing = state.spacing as f32;
//         let mut size = state.rect.size();

//         if let Some(children) = self.children_ref() {
//             let mut expand = Size::default();

//             children
//                 .iter()
//                 .filter(|child| child.node().is_visible())
//                 .enumerate()
//                 .for_each(|(i, child)| {
//                     let child_size = child.calculate_size(Some(self));
//                     let stretch = spacing * i.clamp(0, 1) as f32;

//                     match orientation {
//                         Orientation::Vertical => {
//                             expand.height += child_size.height + stretch;
//                             expand.width = expand.width.max(child_size.width + padding.horizontal() as f32);
//                         }
//                         Orientation::Horizontal => {
//                             expand.height = expand.height.max(child_size.height + padding.vertical() as f32);
//                             expand.width += child_size.width + stretch;
//                         }
//                     }
//                 });

//             match orientation {
//                 Orientation::Vertical => {
//                     expand.height += padding.vertical() as f32;
//                 },
//                 Orientation::Horizontal => {
//                     expand.width += padding.horizontal() as f32;
//                 },
//             }

//             size = expand;
//         }

//         size = size
//             .adjust_on_min_constraints(state.min_width, state.min_height)
//             .adjust_on_max_constraints(state.max_width, state.max_height);

//         let aspect_ratio = match state.image_aspect_ratio {
//             AspectRatio::Defined(n, d) => Some((n, d).into()),
//             AspectRatio::Source => node.borrow()
//                 .background_paint
//                 .aspect_ratio(),
//             AspectRatio::Undefined => None,
//         };

//         if let Some(fraction) = aspect_ratio {
//             match parent {
//                 Some(parent) if parent
//                     .node_ref()
//                     .unwrap()
//                     .upgrade()
//                     .borrow()
//                     .orientation
//                     .is_vertical() => size.adjust_height_with_fraction(fraction),
//                 _ => size.adjust_width_with_fraction(fraction),
//             }
//         }

//         if state.rect.size() == size { return size }

//         drop(state);

//         let mut state = node.borrow_mut();
//         state.rect.set_size(size);
//         state.flag.set_dirty(true);

//         size
//     }
// }

pub(crate) struct LayoutRules {
    pub(crate) orientation: Orientation,
    pub(crate) align_h: AlignH,
    pub(crate) align_v: AlignV,
    pub(crate) padding: Padding,
    pub(crate) spacing: u8,
}

/// Unit to calculate size
pub enum Unit {
    /// fixed unit
    Fixed(f32),
    Grow,
    FitToChild,
}

impl Default for Unit {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Unit {
    fn is_grow(&self) -> bool {
        matches!(self, Self::Grow)
    }
}

pub struct Border {
    paint: Paint,
    width: u32,
}

pub(crate) struct State {
    pub(crate) ptr: SparseIndices<WidgetId>,
    pub(crate) entities: Vec<WidgetId>,

    pub(crate) transform: Vec<Matrix3x2>,
    pub(crate) rect: Vec<Rect>,
    pub(crate) width: Vec<Unit>,
    pub(crate) height: Vec<Unit>,

    // pub(crate) min_width: Vec<Option<f32>>,
    // pub(crate) max_width: Vec<Option<f32>>,

    // pub(crate) min_height: Vec<Option<f32>>,
    // pub(crate) max_height: Vec<Option<f32>>,

    pub(crate) flag: Vec<Flag>,
    pub(crate) background: Vec<Paint>,
    pub(crate) shape: Vec<Shape>,
    pub(crate) corner_radius: Vec<CornerRadius>,

    pub(crate) border: Array<WidgetId, Border>,
    pub(crate) rules: Array<WidgetId, LayoutRules>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            ptr: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),

            transform: Vec::with_capacity(capacity),
            rect: Vec::with_capacity(capacity),
            width: Vec::with_capacity(capacity),
            height: Vec::with_capacity(capacity),

            // min_width: Vec::with_capacity(capacity),
            // max_width: Vec::with_capacity(capacity),
            // min_height: Vec::with_capacity(capacity),
            // max_height: Vec::with_capacity(capacity),

            flag: Vec::with_capacity(capacity),
            background: Vec::with_capacity(capacity),
            shape: Vec::with_capacity(capacity),
            corner_radius: Vec::with_capacity(capacity),

            border: Array::default(),
            rules: Array::default(),
        }
    }

    pub(crate) fn insert_default_state(&mut self, id: &WidgetId) {
        self.ptr.resize_if_needed(id);
        self.entities.push(*id);

        self.transform.push(Matrix3x2::identity());
        self.rect.push(Default::default());
        self.width.push(Default::default());
        self.height.push(Default::default());

        // self.min_width.push(Default::default());
        // self.max_width.push(Default::default());
        // self.min_height.push(Default::default());
        // self.max_height.push(Default::default());

        self.flag.push(Default::default());
        self.background.push(Paint::Color(Rgba::RED));
        self.shape.push(Shape::RoundedRect);
        self.corner_radius.push(CornerRadius::default());
    }

    pub(crate) fn get_flag(&self, id: &WidgetId) -> Option<&Flag> {
        self.ptr.with(id, |index| self.flag.get(index))?
    }

    pub(crate) fn get_flag_mut(&mut self, id: &WidgetId) -> Option<&mut Flag> {
        self.ptr.with(id, |index| self.flag.get_mut(index))?
    }

    pub(crate) fn get_rect(&self, id: &WidgetId) -> Option<&Rect> {
        self.ptr.with(id, |index| self.rect.get(index))?
    }

    pub(crate) fn get_transform(&self, id: &WidgetId) -> Option<&Matrix3x2> {
        self.ptr.with(id, |index| self.transform.get(index))?
    }

    pub(crate) fn set_position(&mut self, id: &WidgetId, position: Vec2f) {
        if let Some(rect) = self.ptr.with(id, |index| &mut self.rect[index]) {
            rect.x = position.x;
            rect.y = position.y;
        }
    }

    pub(crate) fn set_width(&mut self, id: &WidgetId, width: Unit) {
        if let Some(w) = self.ptr.with(id, |index| &mut self.width[index]) {
            *w = width
        }
    }

    pub(crate) fn set_height(&mut self, id: &WidgetId, height: Unit) {
        if let Some(h) = self.ptr.with(id, |index| &mut self.height[index]) {
            *h = height
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.rect.len()
    }

    pub(crate) fn remove(&mut self, id: &WidgetId) {
        if self.len() == 0 { return }

        if let Some(index) = self.ptr.get_data_index(id) {
            let last = self.entities.last().unwrap();

            self.ptr.set_index(last.index(), index);
            self.ptr.set_null(id);
            self.entities.swap_remove(index);

            self.rect.swap_remove(index);
            self.width.swap_remove(index);
            self.height.swap_remove(index);

            // self.min_width.swap_remove(index);
            // self.max_width.swap_remove(index);
            // self.min_height.swap_remove(index);
            // self.max_height.swap_remove(index);

            self.flag.swap_remove(index);
            self.background.swap_remove(index);
            self.shape.swap_remove(index);
            self.corner_radius.swap_remove(index);
        }
    }

    pub(crate) fn set_visible(&mut self, tree: &Tree<WidgetId>, id: &WidgetId, visible: bool) {
        tree.iter_depth(id)
            .for_each(|member| {
                if let Some(flag) = self.ptr.with(member, |index| &mut self.flag[index]) {
                    flag.set_hidden(!visible);
                }
            });
    }

    pub(crate) fn calculate_layout(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
        self.update_fixed_unit();

        self.calculate_size(tree, start);

        // self.update_constraints();

        self.update_growth_unit(tree, start);

        self.calculate_position(tree, start);
    }

    pub(crate) fn update_fixed_unit(&mut self) {
        self.size
            .iter_mut()
            .zip(self.width.iter().zip(&self.height))
            .zip(&self.flag)
            .filter(|(_, flag)| flag.is_visible())
            .for_each(|((size, (width, height)), _)| {
                if let Unit::Fixed(w) = width { size.width = *w }
                if let Unit::Fixed(h) = height { size.height = *h }
            });
    }

    // pub(crate) fn update_constraints(&mut self) {
    //     self.size
    //         .iter_mut()
    //         .zip(self.min_width.iter().zip(&self.max_width))
    //         .zip(self.min_height.iter().zip(&self.max_height))
    //         .zip(&self.flag)
    //         .filter(|(_, flag)| flag.is_visible())
    //         .for_each(|(((size, (min_w, max_w)), (min_h, max_h)), _)| {
    //             *size = size
    //                 .adjust_on_min_constraints(*min_w, *min_h)
    //                 .adjust_on_max_constraints(*max_w, *max_h);
    //         });
    // }

    pub(crate) fn update_growth_unit(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
        tree.iter_depth(start)
            .filter(|id| self.ptr.with(*id, |index| &self.flag[index]).is_some_and(|flag| flag.is_visible()))
            .for_each(|id| {
                if let Some(rules) = self.rules.get(id) {
                    let size = self.ptr.with(id, |index| &self.size[index]).unwrap();

                    let (rem_w, rem_h) = tree.iter_children(id)
                        .flat_map(|child| self.ptr.with(child, |index| &self.size[index]))
                        .fold((size.width, size.height), |(w, h), cs| {
                            match rules.orientation {
                                Orientation::Horizontal => (w - cs.width, h),
                                Orientation::Vertical => (w, h - cs.height),
                            }
                        });

                    let to_grow_w = tree.iter_children(id)
                        .filter(|child| {
                            self.ptr
                                .with(*child, |index| &self.width[index])
                                .is_some_and(|unit| unit.is_grow())
                        })
                        .collect::<Vec<_>>();

                    let to_grow_h = tree.iter_children(id)
                        .filter(|child| {
                            self.ptr
                                .with(*child, |index| &self.height[index])
                                .is_some_and(|unit| unit.is_grow())
                        })
                        .collect::<Vec<_>>();

                    let count_w = to_grow_w.len() as f32;
                    let count_h = to_grow_h.len() as f32;

                    to_grow_w.iter().for_each(|child| {
                        let cs = self.ptr.with(*child, |index| &mut self.size[index]).unwrap();
                        match rules.orientation {
                            Orientation::Horizontal => cs.width += rem_w / count_w,
                            Orientation::Vertical => cs.width = rem_w,
                        }
                    });

                    to_grow_h.iter().for_each(|child| {
                        let cs = self.ptr.with(*child, |index| &mut self.size[index]).unwrap();
                        match rules.orientation {
                            Orientation::Horizontal => cs.height = rem_h,
                            Orientation::Vertical => cs.height += rem_h / count_h,
                        }
                    });
                }
            });
    }

    fn calculate_size(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) -> Size {
        let mut size = Size::default();

        if self.ptr
            .with(start, |index| &self.flag[index])
            .is_some_and(|flag| flag.is_hidden()) { return size }

        if let Some(rules) = self.rules.get(start) {
            let orientation = rules.orientation;
            let padding = rules.padding;
            let spacing = rules.spacing;

            let child_count = tree.iter_children(start)
                .map(|child| {
                    let child_size = self.calculate_size(tree, child);
                    match orientation {
                        Orientation::Horizontal => {
                            size.width += child_size.width;
                            size.height = size.height.max(child_size.height);
                        },
                        Orientation::Vertical => {
                            size.height += child_size.height;
                            size.width = size.width.max(child_size.width);
                        },
                    }
                })
                .count();

            match orientation {
                Orientation::Horizontal => size.width += spacing as f32 * (child_count - 1) as f32,
                Orientation::Vertical => size.height += spacing as f32 * (child_count - 1) as f32,
            }

            size.width += padding.horizontal() as f32;
            size.height += padding.vertical() as f32;
        }

        if let Some(this_size) = self.ptr.with(start, |index| &mut self.size[index]) {
            *this_size = size;
        }

        size
    }

    pub(crate) fn calculate_position(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
        tree.iter_depth(start)
            .filter(|id| {
                self.ptr
                    .with(*id, |index| &self.flag[index])
                    .is_some_and(|flag| flag.is_visible())
            })
            .for_each(|id| {
                if let Some(parent) = tree.get_parent(id)
                    && let Some(rules) = self.rules.get(parent)
                {
                    let prev_pos_size = tree.get_prev_sibling(id)
                        .and_then(|prev| {
                            let pos = self.ptr.with(prev, |index| &self.position[index]).copied();
                            let size = self.ptr.with(prev, |index| &self.size[index]).copied();
                            pos.zip(size)
                        });

                    let parent_pos = *self.ptr.with(parent, |index| &self.position[index]).unwrap();
                    let pos = self.ptr.with(id, |index| &mut self.position[index]).unwrap();

                    let orientation = rules.orientation;
                    let spacing = rules.spacing;
                    let padding = rules.padding;

                    if let Some((p, s)) = prev_pos_size {
                        match orientation {
                            Orientation::Horizontal => {
                                pos.x = p.x + s.width + spacing as f32;
                                pos.y = p.y;
                            },
                            Orientation::Vertical => {
                                pos.x = p.x;
                                pos.y = p.y + s.height + spacing as f32;
                            },
                        }
                    } else {
                        pos.x = parent_pos.x + padding.left as f32;
                        pos.y = parent_pos.y + padding.top as f32;
                    }
                }
            });
    }

    pub(crate) fn update_alignment(&mut self) {}

    pub(crate) fn render(&self, renderer: &mut Renderer) {
        let mut scene = renderer.scene();
        self.entities.iter()
            .zip(&self.position)
            .zip(&self.size)
            .zip(&self.transform)
            .zip(&self.background)
            .zip(&self.corner_radius)
            .zip(&self.shape)
            .zip(&self.flag)
            .for_each(|(((((((id, position), size), transform), background), corner_radius), shape), flag)| {
                if flag.is_visible() {
                    let (border_paint, border_width) = self.border
                        .get(id)
                        .map(|border| {
                            (border.paint.as_paint_ref(), border.width)
                        })
                        .unwrap_or((background.as_paint_ref(), 0));
                    let draw_args = DrawArgs {
                        position,
                        size,
                        transform,
                        background_paint: &background.as_paint_ref(),
                        border_paint: &border_paint,
                        border_width: &border_width,
                        shape,
                        corner_radius,
                    };
                    scene.draw(draw_args);
                }
            });
    }
}
