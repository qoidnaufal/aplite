use aplite_types::{Size, Rect, Matrix3x2, Paint, CornerRadius};
use aplite_storage::Tree;
use aplite_renderer::Shape;

use crate::widget::{Widget, WidgetId};
use crate::state::{AspectRatio, Flag};
use crate::layout::*;
use crate::cursor::Cursor;

#[derive(Default)]
pub(crate) struct ViewStorage {
    pub(crate) tree: Tree<WidgetId, ()>,
    pub(crate) rect: Vec<Option<Rect>>,
    // pub(crate) rotation: Vec<f32>, // in radians
    pub(crate) transform: Vec<Option<Matrix3x2>>,
    pub(crate) min_width: Vec<Option<f32>>,
    pub(crate) min_height: Vec<Option<f32>>,
    pub(crate) max_width: Vec<Option<f32>>,
    pub(crate) max_height: Vec<Option<f32>>,
    pub(crate) align_v: Vec<Option<AlignV>>,
    pub(crate) align_h: Vec<Option<AlignH>>,
    pub(crate) orientation: Vec<Option<Orientation>>,
    pub(crate) padding: Vec<Option<Padding>>,
    pub(crate) spacing: Vec<Option<u8>>,
    // pub(crate) z_index: Vec<Option<u8>>,
    pub(crate) image_aspect_ratio: Vec<Option<AspectRatio>>,
    pub(crate) shape: Vec<Option<Shape>>,
    pub(crate) corner_radius: Vec<Option<CornerRadius>>,
    pub(crate) border_width: Vec<Option<f32>>,
    pub(crate) background_paint: Vec<Option<Paint>>,
    pub(crate) border_paint: Vec<Option<Paint>>,
    pub(crate) flag: Vec<Option<Flag>>,
}

impl ViewStorage {
    pub(crate) fn new_id(&mut self) -> WidgetId {
        self.rect.push(None);
        // self.rotation.push(None);
        self.transform.push(None);
        self.min_width.push(None);
        self.min_height.push(None);
        self.max_width.push(None);
        self.max_height.push(None);
        self.align_v.push(None);
        self.align_h.push(None);
        self.orientation.push(None);
        self.padding.push(None);
        self.spacing.push(None);
        // self.z_index.push(None);
        self.image_aspect_ratio.push(None);
        self.shape.push(None);
        self.corner_radius.push(None);
        self.border_width.push(None);
        self.background_paint.push(None);
        self.border_paint.push(None);
        self.flag.push(None);
        self.tree.insert(())
    }

    pub(crate) fn remove(&mut self, id: WidgetId) {
        let mut removed = self.tree.remove(id);
        removed.drain(..)
            .for_each(|id| {
                self.rect[id.index()] = None;
                // self.rotation[id.index()] = None;
                self.transform[id.index()] = None;
                self.min_width[id.index()] = None;
                self.min_height[id.index()] = None;
                self.max_width[id.index()] = None;
                self.max_height[id.index()] = None;
                self.align_v[id.index()] = None;
                self.align_h[id.index()] = None;
                self.orientation[id.index()] = None;
                self.padding[id.index()] = None;
                self.spacing[id.index()] = None;
                // self.z_index[id.index()] = None;
                self.image_aspect_ratio[id.index()] = None;
                self.shape[id.index()] = None;
                self.corner_radius[id.index()] = None;
                self.border_width[id.index()] = None;
                self.background_paint[id.index()] = None;
                self.border_paint[id.index()] = None;
                self.flag[id.index()] = None;
            });
    }

    pub(crate) fn filter_visible(&self) -> impl Iterator<Item = WidgetId> {
        self.tree
            .iter_data_ref()
            .filter_map(|(id, _)| {
                self.flag[id.index()]
                    .is_some_and(|flag| !flag.is_hidden())
                    .then_some(id)
            })
    }

    pub(crate) fn with_visible<F, U>(&self, mut f: F) -> impl Iterator<Item = U>
    where
        F: FnMut(WidgetId, &Self) -> U,
    {
        self.filter_visible()
            .map(move |id| f(id, self))
    }

    pub(crate) fn with_ref<'a, F, U>(&'a self, id: &WidgetId, f: F) -> Option<U>
    where
        F: FnOnce(StateRef<'a>) -> U,
    {
        self.tree
            .get(id)
            .is_some()
            .then_some(f(StateRef {
                rect: self.rect[id.index()].as_ref(),
                transform: self.transform[id.index()].as_ref(),
                min_width: self.min_width[id.index()].as_ref(),
                min_height: self.min_height[id.index()].as_ref(),
                max_width: self.max_width[id.index()].as_ref(),
                max_height: self.max_height[id.index()].as_ref(),
                align_v: self.align_v[id.index()].as_ref(),
                align_h: self.align_h[id.index()].as_ref(),
                orientation: self.orientation[id.index()].as_ref(),
                padding: self.padding[id.index()].as_ref(),
                spacing: self.spacing[id.index()].as_ref(),
                image_aspect_ratio: self.image_aspect_ratio[id.index()].as_ref(),
                shape: self.shape[id.index()].as_ref(),
                corner_radius: self.corner_radius[id.index()].as_ref(),
                border_width: self.border_width[id.index()].as_ref(),
                background_paint: self.background_paint[id.index()].as_ref(),
                border_paint: self.border_paint[id.index()].as_ref(),
                flag: self.flag[id.index()].as_ref(),
            }))
    }

    pub(crate) fn with_mut<'a, F, U>(&'a mut self, id: &WidgetId, f: F) -> Option<U>
    where
        F: FnOnce(StateMut<'a>) -> U,
    {
        self.tree
            .get(id)
            .is_some()
            .then_some(f(StateMut {
                rect: self.rect[id.index()].as_mut(),
                transform: self.transform[id.index()].as_mut(),
                min_width: self.min_width[id.index()].as_mut(),
                min_height: self.min_height[id.index()].as_mut(),
                max_width: self.max_width[id.index()].as_mut(),
                max_height: self.max_height[id.index()].as_mut(),
                align_v: self.align_v[id.index()].as_mut(),
                align_h: self.align_h[id.index()].as_mut(),
                orientation: self.orientation[id.index()].as_mut(),
                padding: self.padding[id.index()].as_mut(),
                spacing: self.spacing[id.index()].as_mut(),
                image_aspect_ratio: self.image_aspect_ratio[id.index()].as_mut(),
                shape: self.shape[id.index()].as_mut(),
                corner_radius: self.corner_radius[id.index()].as_mut(),
                border_width: self.border_width[id.index()].as_mut(),
                background_paint: self.background_paint[id.index()].as_mut(),
                border_paint: self.border_paint[id.index()].as_mut(),
                flag: self.flag[id.index()].as_mut(),
            }))
    }
}

pub(crate) struct StateRef<'a> {
    pub(crate) rect: Option<&'a Rect>,
    // pub(crate) rotation: Option<&'a f32>, // in radians
    pub(crate) transform: Option<&'a Matrix3x2>,
    pub(crate) min_width: Option<&'a f32>,
    pub(crate) min_height: Option<&'a f32>,
    pub(crate) max_width: Option<&'a f32>,
    pub(crate) max_height: Option<&'a f32>,
    pub(crate) align_v: Option<&'a AlignV>,
    pub(crate) align_h: Option<&'a AlignH>,
    pub(crate) orientation: Option<&'a Orientation>,
    pub(crate) padding: Option<&'a Padding>,
    pub(crate) spacing: Option<&'a u8>,
    // pub(crate) z_index: Option<&'a u8>,
    pub(crate) image_aspect_ratio: Option<&'a AspectRatio>,
    pub(crate) shape: Option<&'a Shape>,
    pub(crate) corner_radius: Option<&'a CornerRadius>,
    pub(crate) border_width: Option<&'a f32>,
    pub(crate) background_paint: Option<&'a Paint>,
    pub(crate) border_paint: Option<&'a Paint>,
    pub(crate) flag: Option<&'a Flag>,
}

pub(crate) struct StateMut<'a> {
    pub(crate) rect: Option<&'a mut Rect>,
    // pub(crate) rotation: Option<&'a mut f32>, // in radians
    pub(crate) transform: Option<&'a mut Matrix3x2>,
    pub(crate) min_width: Option<&'a mut f32>,
    pub(crate) min_height: Option<&'a mut f32>,
    pub(crate) max_width: Option<&'a mut f32>,
    pub(crate) max_height: Option<&'a mut f32>,
    pub(crate) align_v: Option<&'a mut AlignV>,
    pub(crate) align_h: Option<&'a mut AlignH>,
    pub(crate) orientation: Option<&'a mut Orientation>,
    pub(crate) padding: Option<&'a mut Padding>,
    pub(crate) spacing: Option<&'a mut u8>,
    // pub(crate) z_index: Option<&'a mut u8>,
    pub(crate) image_aspect_ratio: Option<&'a mut AspectRatio>,
    pub(crate) shape: Option<&'a mut Shape>,
    pub(crate) corner_radius: Option<&'a mut CornerRadius>,
    pub(crate) border_width: Option<&'a mut f32>,
    pub(crate) background_paint: Option<&'a mut Paint>,
    pub(crate) border_paint: Option<&'a mut Paint>,
    pub(crate) flag: Option<&'a mut Flag>,
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    pub(crate) widget: Box<dyn Widget>,
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub(crate) fn detect_hover(&self, cursor: &Cursor) -> Option<&Box<dyn Widget>> {
        let mut current = &self.widget;

        while let Some(children) = current.children_ref() {
            if let Some(hovered) = children.visible_boxed()
                .find_map(|child| {
                    child.node_ref()
                        .upgrade()
                        .borrow()
                        .rect
                        .contains(cursor.hover.pos)
                        .then_some(child)
                }) {
                current = hovered
            } else {
                break
            }
        }

        current
            .node_ref()
            .is_hoverable()
            .then_some(current)
    }

    pub(crate) fn calculate_layout(&self, bound: aplite_types::Rect) {
        let window_widget = crate::widget::WindowWidget::new(bound);
        let mut cx = LayoutCx::new(&window_widget);
        self.widget.layout(&mut cx);

        let mut current = self.widget.as_ref();

        while let Some(children) = current.children_ref() {
            let mut cx = LayoutCx::new(current);

            for child in children.visible_ref() {
                child.layout(&mut cx);
            }

            for child in children.visible_ref() {
                current = child;
            }
        }
    }

    pub(crate) fn find_parent(&self, id: &WidgetId) -> Option<&Box<dyn Widget>> {
        if self.widget.as_ref().id() == id { return None }

        let mut current = &self.widget;

        while let Some(children) = current.children_ref() {
            if children.iter().any(|child| child.id() == id) { break }

            children.all_boxed().for_each(|child| current = child);
        }

        current.id().ne(id).then_some(current)
    }

    pub(crate) fn find_visible(&self, id: &WidgetId) -> Option<&dyn Widget> {
        let mut current = self.widget.as_ref();

        while let Some(children) = current.children_ref() {
            if current.id() == id { break }

            children
                .visible_ref()
                .for_each(|child| current = child);
        }

        Some(current)
    }

    // fn insert<T: Widget + 'static>(&mut self, parent: &WidgetId, widget: T) {
    //     if let Some(mut p) = self.find_mut(parent)
    //         && let Some(vec) = p.children_mut()
    //     {
    //         vec.push(Box::new(widget));
    //     }
    // }

    // fn remove(&mut self, id: &WidgetId) -> Option<Box<dyn Widget>> {
    //     self.parent_mut(id)
    //         .and_then(|mut parent| {
    //             parent.children_mut()
    //                 .and_then(|children| {
    //                     children.iter()
    //                         .position(|w| w.id() == id)
    //                         .map(|index| children.remove(index))
    //                 })
    //         })
    // }
}

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget + 'static> IntoView for T {
    fn into_view(self) -> View {
        View::new(self)
    }
}

impl<T: Widget + Sized + 'static> Layout for T {}

pub(crate) trait Layout: Widget + Sized + 'static {
    fn calculate_layout(&self, cx: &mut LayoutCx) {
        // self.layout(cx);

        // let mut current = self as &dyn Widget;

        // while let Some(children) = current.children_ref() {
        //     for child in children.visible_ref() {
        //         let mut new_cx = LayoutCx::new(current);
        //         child.layout(&mut new_cx);
        //         current = child;
        //     }
        // }

        if self.layout(cx)
            && let Some(children) = self.children_ref()
        {
            let mut this_cx = LayoutCx::new(self);
            children.iter()
                .for_each(|child| child.calculate_layout(&mut this_cx));
        }
    }

    fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
        let node = self.node_ref().upgrade();
        if node.borrow().flag.is_hidden() { return Size::default() }

        let state = node.borrow();
        let padding = state.padding;
        let orientation = state.orientation;
        let spacing = state.spacing as f32;
        let mut size = state.rect.size();

        if let Some(children) = self.children_ref() {
            let mut expand = Size::default();

            children
                .iter()
                .filter(|child| child.node_ref().is_visible())
                .enumerate()
                .for_each(|(i, child)| {
                    let child_size = child.calculate_size(Some(self));
                    let stretch = spacing * i.clamp(0, 1) as f32;

                    match orientation {
                        Orientation::Vertical => {
                            expand.height += child_size.height + stretch;
                            expand.width = expand.width.max(child_size.width + padding.horizontal() as f32);
                        }
                        Orientation::Horizontal => {
                            expand.height = expand.height.max(child_size.height + padding.vertical() as f32);
                            expand.width += child_size.width + stretch;
                        }
                    }
                });

            match orientation {
                Orientation::Vertical => {
                    expand.height += padding.vertical() as f32;
                },
                Orientation::Horizontal => {
                    expand.width += padding.horizontal() as f32;
                },
            }

            size = expand;
        }

        size = size
            .adjust_on_min_constraints(state.min_width, state.min_height)
            .adjust_on_max_constraints(state.max_width, state.max_height);

        let aspect_ratio = match state.image_aspect_ratio {
            AspectRatio::Defined(n, d) => Some((n, d).into()),
            AspectRatio::Source => node.borrow()
                .background_paint
                .aspect_ratio(),
            AspectRatio::Undefined => None,
        };

        if let Some(fraction) = aspect_ratio {
            match parent {
                Some(parent) if parent
                    .node_ref()
                    .upgrade()
                    .borrow()
                    .orientation
                    .is_vertical() => size.adjust_height_with_fraction(fraction),
                _ => size.adjust_width_with_fraction(fraction),
            }
        }

        if state.rect.size() == size { return size }

        drop(state);

        let mut state = node.borrow_mut();
        state.rect.set_size(size);
        state.flag.set_dirty(true);

        size
    }
}

#[cfg(test)]
mod ptr_test {
    struct PtrWrapper(*const dyn Name);

    impl Name for PtrWrapper {
        fn name(&self) -> &str {
            unsafe {
                self.0.as_ref().unwrap().name()
            }
        }
    }

    trait Caster: Name + Sized + 'static {
        fn get_ptr(&self) -> PtrWrapper {
            PtrWrapper(self as *const dyn Name)
        } 
    }

    impl Name for Box<dyn Name> {
        fn name(&self) -> &str {
            self.as_ref().name()
        }
    }

    impl Caster for Box<dyn Name> {}

    trait Name {
        fn name(&self) -> &str;
    }

    struct MyStruct {
        name: String,
    }

    impl MyStruct {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into()
            }
        }
    }

    impl Name for MyStruct {
        fn name(&self) -> &str {
            &self.name
        }
    }

    struct TraitContainer {
        inner: Box<dyn Name>
    }

    #[test]
    fn ptr() {
        let mystruct = MyStruct::new("one");
        let container = TraitContainer { inner: Box::new(mystruct) };
        let wrapper = container.inner.get_ptr();

        let name = wrapper.name();
        assert_eq!(name, "one");
    }
}
