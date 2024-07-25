use circle::Circle;
use std::f32::consts::SQRT_2;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use winit::event::{Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub mod circle;
mod winit_app;

const TARGET_FRAMETIME: f64 = 1.0/60.0/8.0;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut circles = vec![Circle::new(
        (0.3, 0.9),
        0.1,
        Color::from_rgba8(255, 0, 0, 255),
    )];

    let mut cursor_pos = (0.0, 0.0);
    let mut remainder: f64 = 0.0;
    let mut frame_start = Instant::now();

    let mut app = winit_app::WinitAppBuilder::with_init(|elwt| {
        let window = {
            let window = elwt.create_window(Window::default_attributes());
            Rc::new(window.unwrap())
        };
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        (window, surface)
    })
    .with_event_handler(|state, event, elwt| {
        let (window, surface) = state;
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                    },
            } if window_id == window.id() => {
                cursor_pos = (position.x as f32, position.y as f32);
                if !circles[0].has_physics {
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };
                    let scaling = (width.min(height)) as f32;
                    let x_shift = (((width as f32) - scaling) * 0.5) as f32;
                    let y_shift = (((height as f32) - scaling) * 0.5) as f32;
                    let x = (cursor_pos.0 - x_shift) / scaling;
                    let y = 1.0 - ((cursor_pos.1 - y_shift) / scaling);
                    circles[0].pos = (x, y);
                    circles[0].prev_pos = (x, y);
                    circles[0].has_physics = false;
                }
            }
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    },
            } if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                let scaling = (width.min(height)) as f32;
                let x_shift = (((width as f32) - scaling) * 0.5) as f32;
                let y_shift = (((height as f32) - scaling) * 0.5) as f32;

                if button == MouseButton::Left && state.is_pressed() {
                    let x = (cursor_pos.0 - x_shift) / scaling;
                    let y = 1.0 - ((cursor_pos.1 - y_shift) / scaling);
                    circles[0].pos = (x, y);
                    circles[0].prev_pos = (x, y);
                    circles[0].has_physics = false;
                }
                if button == MouseButton::Left && !state.is_pressed() {
                    circles[0].has_physics = true;
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                window.request_redraw();
                let elapsed = frame_start.elapsed();
                frame_start = Instant::now();
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                // Update physics
                remainder += elapsed.as_secs_f64();
                while remainder >= TARGET_FRAMETIME {
                    update(&mut circles, TARGET_FRAMETIME as f32);
                    remainder -= TARGET_FRAMETIME;
                }

                let mut buffer = surface.buffer_mut().unwrap();

                let mut pixmap = Pixmap::new(width, height).unwrap();
                pixmap.fill(Color::BLACK);

                {
                    let mut paint = Paint::default();
                    paint.anti_alias = true;
                    let scaling = (width.min(height)) as f32;
                    let x_shift = (((width as f32) - scaling) * 0.5) as f32;
                    let y_shift = (((height as f32) - scaling) * 0.5) as f32;

                    for circle in &circles {
                        let x = circle.pos.0 * scaling + x_shift;
                        let y = (1.0 - circle.pos.1) * scaling + y_shift;
                        let circle_path =
                            PathBuilder::from_circle(x, y, circle.radius * scaling).unwrap();

                        paint.set_color(circle.color);
                        pixmap.fill_path(
                            &circle_path,
                            &paint,
                            FillRule::Winding,
                            Transform::default(),
                            None,
                        );
                    }

                    paint.set_color(Color::WHITE);
                    let cage = {
                        let mut builder = PathBuilder::new();
                        builder.move_to(x_shift, y_shift);
                        builder.line_to(x_shift, y_shift + scaling);
                        builder.line_to(x_shift + scaling, y_shift + scaling);
                        builder.line_to(x_shift + scaling, y_shift);
                        builder.move_to(x_shift, y_shift + scaling * 0.5);
                        builder.line_to(x_shift + scaling * 0.5, y_shift + scaling);
                        builder.finish().unwrap()
                    };
                    pixmap.stroke_path(
                        &cage,
                        &paint,
                        &Stroke::default(),
                        Transform::default(),
                        None,
                    );
                }

                let pixiter = pixmap
                    .data()
                    .chunks_exact(4)
                    .map(|arr| u32::from_le_bytes(arr.try_into().unwrap()));

                for (i, pix) in pixiter.enumerate() {
                    buffer[i] = pix;
                }

                buffer.present().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            _ => {}
        }
    });

    event_loop.run_app(&mut app).unwrap();
}

fn update(circles: &mut [Circle], dt: f32) {
    // Apply step with gravity
    for circle in circles.as_mut() {
        circle.step((0.0, -0.8), dt);
    }

    // Handle slope
    {
        let normal = {
            let unnormalized: (f32, f32) = (1.0, 1.0);
            let len = unnormalized.0 * unnormalized.0 + unnormalized.1 * unnormalized.1;
            let len = len.sqrt();
            (unnormalized.0 / len, unnormalized.1 / len)
        };
        for circle in circles.as_mut() {
            let pos = (circle.pos.0-0.5, circle.pos.1);
            let dist = pos.0 * normal.0 + pos.1 * normal.1;
            if dist < circle.radius {
                let fac = -dist+circle.radius;
                let dp = (normal.0*fac, normal.1*fac);
                circle.pos = (circle.pos.0+dp.0, circle.pos.1+dp.1);
            }
        }
    }

    // Restrict every circle to collide with the ground
    for circle in circles.as_mut() {
        if circle.pos.1 < circle.radius {
            circle.pos.1 = circle.radius;
        }
        if circle.pos.0 > 1.0 - circle.radius {
            circle.pos.0 = 1.0 - circle.radius;
        }
        if circle.pos.0 < circle.radius {
            circle.pos.0 = circle.radius;
        }
    }
}
