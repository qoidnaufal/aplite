use shared::{Size, Vector2, Rgba};

use crate::properties::Properties;
use crate::renderer::CornerRadius;
use crate::renderer::Shape;
use crate::tree::Entity;
use crate::tree::NodeId;

use super::layout::Alignment;
use super::layout::HAlign;
use super::layout::Orientation;
use super::layout::Padding;
use super::layout::VAlign;

#[derive(Clone, Copy)]
pub(crate) struct LayoutData {
    position: Vector2<u32>,
    size: Size<u32>,
}

impl LayoutData {
    pub(crate) fn new(position: Vector2<u32>, size: Size<u32>) -> Self {
        Self { position, size }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutRules {
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    alignment: Alignment,
    orientation: Orientation,
    padding: Padding,
    spacing: u32,
}

impl LayoutRules {
    pub(crate) fn new(prop: &Properties) -> Self {
        Self {
            min_width: prop.min_width(),
            min_height: prop.min_height(),
            max_width: prop.max_width(),
            max_height: prop.max_height(),
            orientation: prop.orientation(),
            alignment: prop.alignment(),
            padding: prop.padding(),
            spacing: prop.spacing(),
        }
    }

    pub(crate) fn min_width(&self) -> Option<u32> { self.min_width }
    pub(crate) fn min_height(&self) -> Option<u32> { self.min_height }
    pub(crate) fn max_width(&self) -> Option<u32> { self.max_width }
    pub(crate) fn max_height(&self) -> Option<u32> { self.max_height }
    pub(crate) fn alignment(&self) -> Alignment { self.alignment }
    pub(crate) fn orientation(&self) -> Orientation { self.orientation }
    pub(crate) fn padding(&self) -> Padding { self.padding }
    pub(crate) fn spacing(&self) -> u32 { self.spacing }
}

impl LayoutRules {
    // fn inner_space(&self, size: Size<u32>) -> Size<u32> {
    //     (size.width() - self.padding.horizontal(),
    //     size.height() - self.padding.vertical()).into()
    // }

    fn offset_x(&self, layout_data: &LayoutData) -> u32 {
        let pl = self.padding.left();
        let pr = self.padding.right();

        match self.alignment.h_align {
            HAlign::Left => layout_data.position.x() - (layout_data.size.width() / 2) + pl,
            HAlign::Center => {
                if pl >= pr {
                    layout_data.position.x() + (pl - pr) / 2
                } else {
                    layout_data.position.x() - (pr - pl) / 2
                }
            }
            HAlign::Right => layout_data.position.x() + (layout_data.size.width() / 2) - pr
        }
    }

    fn offset_y(&self, layout_data: &LayoutData) -> u32 {
        let pt = self.padding.top();
        let pb = self.padding.bottom();

        match self.alignment.v_align {
            VAlign::Top => layout_data.position.y() - (layout_data.size.height() / 2) + pt,
            VAlign::Middle => {
                if pt >= pb {
                    layout_data.position.y() + (pt - pb) / 2
                } else {
                    layout_data.position.y() - (pb - pt) / 2
                }
            }
            VAlign::Bottom => layout_data.position.y() + (layout_data.size.height() / 2) - pb,
        }
    }

    fn start_x(&self, total_width: u32, len: u32, layout_data: &LayoutData) -> u32 {
        let offset_x = self.offset_x(layout_data);
        let stretch = self.spacing * (len - 1);
        let final_width = total_width + stretch;

        match self.orientation {
            Orientation::Vertical => offset_x,
            Orientation::Horizontal => {
                match self.alignment.h_align {
                    HAlign::Left => offset_x,
                    HAlign::Center => offset_x - (final_width / 2),
                    HAlign::Right => offset_x - final_width,
                }
            }
        }
    }

    fn start_y(&self, total_height: u32, len: u32, layout_data: &LayoutData) -> u32 {
        let offset_y = self.offset_y(layout_data);
        let stretch = self.spacing * (len - 1);
        let final_height = total_height + stretch;

        match self.orientation {
            Orientation::Vertical => {
                match self.alignment.v_align {
                    VAlign::Top => offset_y,
                    VAlign::Middle => offset_y - (final_height / 2),
                    VAlign::Bottom => offset_y - final_height,
                }
            }
            Orientation::Horizontal => offset_y,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StyleData {
    hover_color: Option<Rgba<u8>>,
    click_color: Option<Rgba<u8>>,
    fill_color: Rgba<u8>,
    stroke_color: Rgba<u8>,
    shape: Shape,
    corners: CornerRadius,
    rotation: f32,
    stroke_width: u32,
    texture_id: i32,
    is_dragable: bool,
}

impl StyleData {
    pub(crate) fn new(prop: &Properties) -> Self {
        Self {
            hover_color: prop.hover_color(),
            click_color: prop.click_color(),
            fill_color: prop.fill_color(),
            stroke_color: prop.stroke_color(),
            shape: prop.shape(),
            corners: prop.corners(),
            rotation: prop.rotation(),
            stroke_width: prop.stroke_width(),
            texture_id: prop.texture_id(),
            is_dragable: prop.is_dragable(),
        }
    }
}

pub(crate) struct Data {
    layout: Vec<LayoutData>,
    rules: Vec<LayoutRules>,
    style: Vec<StyleData>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            layout: Vec::with_capacity(1024),
            rules: Vec::with_capacity(1024),
            style: Vec::with_capacity(1024),
        }
    }
}

impl Data {
    pub(crate) fn insert(&mut self, prop: &Properties) {
        self.layout.push(LayoutData::new(prop.pos(), prop.size()));
        self.rules.push(LayoutRules::new(prop));
        self.style.push(StyleData::new(prop));
    }
}

// layout
impl Data {
    pub(crate) fn get_layout_data(&self, node_id: &NodeId) -> &LayoutData {
        &self.layout[node_id.index()]
    }

    pub(crate) fn get_layout_data_mut(&mut self, node_id: &NodeId) -> &mut LayoutData {
        &mut self.layout[node_id.index()]
    }
}

// rules
impl Data {
    pub(crate) fn get_rules(&self, node_id: &NodeId) -> &LayoutRules {
        &self.rules[node_id.index()]
    }

    pub(crate) fn get_rules_mut(&mut self, node_id: &NodeId) -> &mut LayoutRules {
        &mut self.rules[node_id.index()]
    }
}
