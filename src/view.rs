use aplite_renderer::Renderer;
use aplite_types::Size;

use crate::widget::{Widget, WidgetId};
use crate::state::{ViewNode, AspectRatio};
use crate::layout::*;
use crate::cursor::Cursor;

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget + 'static> IntoView for T {
    fn into_view(self) -> View {
        View::new(self)
    }
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    inner: Box<dyn Widget>
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl Widget for View {
    fn node(&self) -> ViewNode {
        self.inner.node()
    }

    fn id(&self) -> WidgetId {
        self.inner.id()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.inner.children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.inner.children_mut()
    }
}

impl Widget for Box<dyn IntoView> {
    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.as_mut().children_mut()
    }
}

impl std::fmt::Debug for Box<dyn Widget> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.node().borrow().name;
        let name = name.is_empty()
            .then_some(std::any::type_name::<Self>())
            .unwrap_or(name);

        f.debug_struct(name)
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(&vec![]))
            .finish()
    }
}

impl<T: Widget + Sized> Render for T {}

pub(crate) trait Render: Widget + Sized {
    fn render(&self, renderer: &mut Renderer) {
        if self.draw(renderer) {
            if let Some(children) = self.children_ref() {
                children.iter().for_each(|w| w.render(renderer));
            }
        }
    }
}

impl<T: Widget + Sized + 'static> Layout for T {}

pub(crate) trait Layout: Widget + Sized + 'static {
    // TODO: include calculating the size too
    fn calculate_layout(&self, cx: &mut LayoutCx) {
        if self.layout(cx) {
            if let Some(children) = self.children_ref() {
                let mut this_cx = LayoutCx::new(self);
                children.iter()
                    .for_each(|child| child.calculate_layout(&mut this_cx));
            }
        }
    }

    fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
        let node = self.node();

        if node.borrow().hide { return Size::default() }

        let state = node.borrow();
        let padding = state.padding;
        let orientation = state.orientation;
        let spacing = state.spacing;
        let mut size = state.rect.size();

        if let Some(children) = self.children_ref() {
            let mut expand = Size::default();

            children.iter().for_each(|child| {
                let child_size = child.calculate_size(Some(self));
                match orientation {
                    Orientation::Vertical => {
                        expand.height += child_size.height;
                        expand.width = expand.width.max(child_size.width + padding.horizontal());
                    }
                    Orientation::Horizontal => {
                        expand.height = expand.height.max(child_size.height + padding.vertical());
                        expand.width += child_size.width;
                    }
                }
            });

            let child_len = children.len() as f32;
            let stretch = spacing * (child_len - 1.);

            match orientation {
                Orientation::Vertical => {
                    expand.height += padding.vertical() + stretch;
                },
                Orientation::Horizontal => {
                    expand.width += padding.horizontal() + stretch;
                },
            }

            size = expand;
        }

        if let AspectRatio::Defined(tuple) = state.image_aspect_ratio {
            match parent {
                Some(parent) if parent
                    .node()
                    .borrow()
                    .orientation
                    .is_vertical() => size.adjust_height_aspect_ratio(tuple.into()),
                _ => size.adjust_width_aspect_ratio(tuple.into()),
            }
        }

        let final_size = size
            .adjust_on_min_constraints(state.min_width, state.min_height)
            .adjust_on_max_constraints(state.max_width, state.max_height);

        drop(state);

        let mut state = node.borrow_mut();
        state.rect.set_size(final_size);

        final_size
    }

    fn mouse_hover(&self, cursor: &Cursor) -> Option<*const dyn Widget> {
        if self.node().borrow().hide { return None }

        if let Some(children) = self.children_ref() {
            let hovered = children.iter()
                .find_map(|child| child.mouse_hover(cursor));

            if hovered.is_some() {
                return hovered;
            }
        }

        self.node()
            .borrow()
            .rect
            .contains(cursor.hover.pos)
            .then_some(self as *const dyn Widget)
    }

    // fn find(&self, id: &WidgetId) -> Option<Box<&dyn Widget>> {
    //     if self.id() == id {
    //         return Some(Box::new(self))
    //     }

    //     self.children_ref().and_then(|vec| {
    //         vec.iter().find_map(|w| w.find(id))
    //     })
    // }

    // fn find_mut(&mut self, id: &WidgetId) -> Option<Box<&mut dyn Widget>> {
    //     if self.id() == id {
    //         return Some(Box::new(self))
    //     }
    //     self.children_mut().and_then(|vec| {
    //         vec.iter_mut().find_map(|w| w.find_mut(id))
    //     })
    // }

    // fn parent_mut(&mut self, id: &WidgetId) -> Option<Box<&mut dyn Widget>> {
    //     if let Some(children) = self.children_ref()
    //         && children
    //             .iter()
    //             .any(|w| w.id() == id)
    //     {
    //         return Some(Box::new(self))
    //     }

    //     self.children_mut()
    //         .and_then(|vec| {
    //             vec.iter_mut()
    //                 .find_map(|w| w.parent_mut(id))
    //         })
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

    // fn insert<T: Widget + 'static>(&mut self, parent: &WidgetId, widget: T) {
    //     if let Some(mut p) = self.find_mut(parent)
    //         && let Some(vec) = p.children_mut()
    //     {
    //         vec.push(Box::new(widget));
    //     }
    // }
}

#[cfg(test)]
mod ptr_test {
    struct PtrWrapper(*const dyn Name);

    impl PtrWrapper {
        fn name(&self) -> Option<&str> {
            unsafe {
                self.0.as_ref().map(|caster| caster.name())
            }
        }
    }

    trait Caster: Name + Sized + 'static {
        fn get_ptr(&self) -> PtrWrapper {
            PtrWrapper(self as *const dyn Name)
        } 
    }

    impl<T: Name + 'static> Caster for T {}

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

    #[test]
    fn ptr() {
        let mystruct = MyStruct::new("one");
        let wrapper = mystruct.get_ptr();

        let name = wrapper.name();
        assert!(name.is_some());
        eprintln!("{}", name.unwrap());
    }
}
