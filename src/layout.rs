use crate::{
    app::CONTEXT,
    color::Rgb,
    shapes::Vertex,
    types::{cast_slice, tan, Vector3}
};

pub struct Layout {
    pub data: Vec<u8>,
}

impl Layout {
    pub fn new() -> Self {
        Self { data: Default::default() }
    }

    fn and(&mut self, triangle: Triangle) -> &mut Self {
        let new_data = triangle.data();
        self.data.extend_from_slice(cast_slice(&new_data).unwrap());
        self
    }
}

pub struct Triangle {
    pub pos: Vector3<u32>,
    pub width: u32,
    pub height: u32,
    pub color: Rgb<u8>,
}

impl Triangle {
    pub const INDICES: [u16; 3] = [0, 1, 2];

    pub fn new(pos: Vector3<u32>, width: u32, height: u32, color: Rgb<u8>) -> Self {
        Self { pos, width, height, color }
    }

    pub fn data(&self) -> Vec<Vertex> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vector3 { x: x_pos + x_center, y: y_pos, z: self.pos.z as _ };
        let l = Vector3 { x: x_pos, y: y_pos + height, z: self.pos.z as _ };
        let r = Vector3 { x: x_pos + width, y: y_pos + height, z: self.pos.z as _ };

        [
            Vertex { position: t, color: self.color.into() },
            Vertex { position: l, color: self.color.into() },
            Vertex { position: r, color: self.color.into() },
        ].to_vec()
    }

    pub fn is_hovered(&self) -> bool {
        let (window_size, cursor) = CONTEXT.with_borrow(|ctx| (ctx.window_size, ctx.cursor));

        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);

        let x_cursor = ((cursor.position.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.position.y / window_size.height as f32)) * 2.0;

        let cursor_tan = tan(x_pos + x_center - x_cursor, y_pos - y_cursor);
        let triangle_tan = tan(x_center, height);

        (y_pos + height..y_pos).contains(&y_cursor)
            && (x_pos..x_pos + width).contains(&x_cursor)
            && cursor_tan >= triangle_tan
    }

    pub fn set_color<F: FnMut(&mut Rgb<u8>)>(&mut self, mut f: F) {
        f(&mut self.color);
    }

    pub fn set_position(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);

        let delta_x = cursor.position.x - cursor.click.cur.x;
        let delta_y = cursor.position.y - cursor.click.cur.y;

        self.pos.x = (cursor.click.obj.x as f32 + delta_x * 2.) as u32;
        self.pos.y = (cursor.click.obj.y as f32 + delta_y * 2.) as u32;
    }
}

