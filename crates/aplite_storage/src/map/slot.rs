use core::mem;

enum Content<T> {
    Occupied(T),
    Next(u32),
}

pub(crate) struct Slot<T> {
    content: Content<T>,
    pub(crate) version: u32,
}

impl<T> Slot<T> {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            content: Content::Next(1),
            version: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn with_data(data: T) -> Self {
        Self {
            content: Content::Occupied(data),
            version: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn occupy(&mut self, data: T) -> Option<u32> {
        match self.content {
            Content::Occupied(_) => None,
            Content::Next(next) => {
                self.content = Content::Occupied(data);
                Some(next)
            },
        }
    }

    #[inline(always)]
    pub(crate) fn try_replace_with(&mut self, next: u32) -> Option<T> {
        match mem::replace(&mut self.content, Content::Next(next)) {
            Content::Occupied(val) => {
                self.version += 1;
                Some(val)
            },
            Content::Next(actual) => {
                self.content = Content::Next(actual);
                None
            },
        }
    }

    #[inline(always)]
    pub(crate) fn get_content(&self) -> Option<&T> {
        match &self.content {
            Content::Occupied(val) => Some(val),
            Content::Next(_) => None,
        }
    }

    #[inline(always)]
    pub(crate) fn get_content_mut(&mut self) -> Option<&mut T> {
        match &mut self.content {
            Content::Occupied(val) => Some(val),
            Content::Next(_) => None,
        }
    }
}
