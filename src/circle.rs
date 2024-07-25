pub struct Circle {
    pub pos: (f32, f32),
    pub prev_pos: (f32, f32),
    pub radius: f32,
    pub color: tiny_skia::Color,
    pub has_physics: bool,
}

impl Circle {
    pub fn new(pos: (f32, f32), radius: f32, color: tiny_skia::Color) -> Self {
        Self {
            pos,
            prev_pos: pos,
            radius,
            color,
            has_physics: true,
        }
    }

    pub fn new_with_velocity(
        pos: (f32, f32),
        vel: (f32, f32),
        radius: f32,
        color: tiny_skia::Color,
    ) -> Self {
        let dt = 1.0 / 60.0;
        Self {
            pos: (pos.0 + vel.0 * dt, pos.1 + vel.1 * dt),
            prev_pos: pos,
            radius,
            color,
            has_physics: true,
        }
    }

    pub fn step(&mut self, acc: (f32, f32), dt: f32) {
        if self.has_physics {
            let velocity = (self.pos.0 - self.prev_pos.0, self.pos.1 - self.prev_pos.1);
            self.prev_pos = self.pos;
            self.pos = (
                self.pos.0 + velocity.0 + acc.0 * dt * dt,
                self.pos.1 + velocity.1 + acc.1 * dt * dt,
            );
        }
    }
}
