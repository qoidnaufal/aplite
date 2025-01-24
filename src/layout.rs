use std::collections::HashMap;
use math::Size;
use crate::NodeId;

pub struct LayoutCtx {
    size: Size<u32>,
    children: HashMap<NodeId, Vec<NodeId>>,
    parent: HashMap<NodeId, Option<NodeId>>,
}
