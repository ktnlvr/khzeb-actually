mod renderer;
use renderer::{instance::BatchInstance, Renderer};

use glam::Vec2;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = Renderer::new(&window);

    let arc_batch = renderer.create_batch(0x100);

    let batches = [
        BatchInstance::new()
            .with_position_f32(Vec2::new(1., 1.))
            .with_texture_idx(4),
        BatchInstance::new()
            .with_tint(0xFF00FFFF)
            .with_texture_idx(5),
    ];

    for batch in batches {
        arc_batch.push_unchecked(batch);
    }

    arc_batch.flush(renderer.transfer_queue());

    event_loop
        .run(|event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => {
                    renderer.resize(*new_size);
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::RedrawRequested => {
                    renderer.render();
                    window.request_redraw();
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
