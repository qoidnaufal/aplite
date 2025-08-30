use aplite_types::Size;
// use aplite_renderer::Scene;

use crate::widget::Widget;
use crate::state::{AspectRatio, WidgetId};
use crate::layout::*;
use crate::cursor::Cursor;

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
                        .unwrap()
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
            .node()
            .is_hoverable()
            .then_some(current)
    }

    // pub(crate) fn calculate_layout(&self, bound: aplite_types::Rect) {
    //     let window_widget = crate::widget::WindowWidget::new(bound);
    //     let mut cx = LayoutCx::new(&window_widget);

    //     let mut current = self.widget.as_ref();

    //     loop {
    //         if current.layout(&mut cx) {
    //             if let Some(children) = current.children_ref() {
    //                 cx = LayoutCx::new(current);

    //                 for child in children.all_ref() {
    //                     current = child;
    //                 }
    //             }
    //         } else {
    //             break
    //         }
    //     }
    // }

    pub(crate) fn find_parent(&self, id: &WidgetId) -> Option<&Box<dyn Widget>> {
        if self.widget.as_ref().id() == id { return None }

        let mut current = &self.widget;

        while let Some(children) = current.children_ref() {
            if children.iter().any(|child| child.id() == id) { break }

            children
                .all_boxed()
                .for_each(|child| current = child);
        }

        current
            .id()
            .ne(id)
            .then_some(current)
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

    // pub(crate) fn render(&self, scene: &mut Scene) {
    //     let mut current = self.widget.as_ref();
    // }

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
        if self.layout(cx) && let Some(children) = self.children_ref() {
            let mut this_cx = LayoutCx::new(self);
            children.iter()
                .for_each(|child| child.calculate_layout(&mut this_cx));
        }
    }

    fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
        let node = self.node_ref().unwrap().upgrade();
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
                .filter(|child| child.node().is_visible())
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
                    .unwrap()
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
