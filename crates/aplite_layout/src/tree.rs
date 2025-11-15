use aplite_storage::IndexMap;
use aplite_types::Size;

pub struct LayoutNode {
    size: Size,
}

pub struct Tree {
    nodes: IndexMap<LayoutNode>,
}
