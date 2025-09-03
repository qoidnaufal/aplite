use crate::widget::{Widget, WidgetId};
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
