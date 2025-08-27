#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum Content<T> {
    Occupied(T),
    // contains index of next free slot
    Vacant(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Slot<T> {
    pub(crate) version: u8,
    pub(crate) content: Content<T>,
}

impl<T> Slot<T> {
    #[inline(always)]
    pub(crate) fn get_content(&self) -> Option<&T> {
        match &self.content {
            Content::Occupied(val) => Some(val),
            Content::Vacant(_) => None,
        }
    }

    #[inline(always)]
    pub(crate) fn get_content_mut(&mut self) -> Option<&mut T> {
        match &mut self.content {
            Content::Occupied(val) => Some(val),
            Content::Vacant(_) => None,
        }
    }
}
