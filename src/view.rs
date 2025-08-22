use aplite_types::Size;

use crate::widget::{Widget, ChildrenRef, ChildrenMut};
use crate::state::{NodeRef, WidgetId, AspectRatio};
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

    pub(crate) fn detect_hover(&self, cursor: &Cursor) -> Option<*const dyn Widget> {
        let mut current: *const dyn Widget = self;

        while let Some(children) = current.children_ref() {
            if let Some(hovered) = children.iter()
                .find_map(|child| {
                    let node = child.node_ref().upgrade();

                    (!node.borrow().flag.is_hidden()
                        && node.borrow().rect.contains(cursor.hover.pos))
                            .then_some(child.as_ref() as *const dyn Widget)
                })
            {
                current = hovered
            } else {
                break
            }
        }

        let node = current.node_ref().upgrade();
        node.borrow()
            .flag
            .is_hoverable()
            .then_some(current)
    }

    pub(crate) fn find_parent(&self, id: &WidgetId) -> Option<*const dyn Widget> {
        if self.inner.id() == id { return None }

        let mut current: *const dyn Widget = self;

        while let Some(children) = current.children_ref() {
            if children.iter().any(|child| child.id() == id) { break }
            for child in children
                .iter()
                .map(|child| child.as_ref() as *const dyn Widget)
                .collect::<Vec<_>>()
            {
                current = child;
            }
        }

        current.id().ne(id).then_some(current)
    }

    // fn find(&self, id: &WidgetId) -> Option<*const dyn Widget> {
    //     if self.id() == id {
    //         return Some(self)
    //     }

    //     self.children_ref().and_then(|vec| {
    //         vec.iter().find_map(|w| w.find(id))
    //     })
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

impl Widget for View {
    fn node_ref(&self) -> NodeRef {
        self.inner.node_ref()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.inner.children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        self.inner.children_mut()
    }
}

impl Widget for Box<dyn IntoView> {
    fn node_ref(&self) -> NodeRef {
        self.as_ref().node_ref()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
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
        let name = self.node_ref().upgrade().borrow().name;
        let name = if name.is_empty() { std::any::type_name::<Self>() } else { name };

        f.debug_struct(name)
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(ChildrenRef::from(&vec![])))
            .finish()
    }
}

impl std::fmt::Debug for Box<dyn IntoView> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.node_ref().upgrade().borrow().name;
        let name = if name.is_empty() { std::any::type_name::<Self>() } else { name };

        f.debug_struct(name)
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(ChildrenRef::from(&vec![])))
            .finish()
    }
}

impl<T: Widget + Sized + 'static> Layout for T {}

pub(crate) trait Layout: Widget + Sized + 'static {
    // FIXME: deconstruct the recursion into loop
    fn calculate_layout(&self, cx: &mut LayoutCx) {
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
