mod entity;
mod render;
mod shaders;
mod scene;

use environment::Environment;
use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::ControlFlow;
use gavle::{Viewport, FramebufferDescriptor, DefaultFramebufferDescriptor, LoadOp, Color};
use winit::dpi::PhysicalSize;
use crate::scene::Scene;
use crate::render::Renderer;

/** Function responsible for running the game inside of a given application
 * environment, provided by the [`environment`] crate. */
pub fn run(env: Environment) {
	let Environment {
		window,
		event_loop,
		device,
		mut swap_buffers,
		mut delta_time
	} = env;

	let mut viewport = Viewport {
		x: 0,
		y: 0,
		width: 800,
		height: 600
	};
	let framebuffer = device.default_framebuffer(
		&DefaultFramebufferDescriptor {
			color_load_op: LoadOp::Clear(Color {
				red: 0.0,
				green: 0.0,
				blue: 0.0,
				alpha: 1.0
			}),
			depth_load_op: LoadOp::Clear(f32::INFINITY),
			stencil_load_op: LoadOp::Clear(0)
		});

	let mut scene = Scene::new(800.0 / 600.0);
	let mut renderer = Renderer::new(&device);

	let _ = (delta_time)();

	let mut direction = 0.0f32;
	let mut angle = std::f32::consts::FRAC_PI_4;

	event_loop.run(move |event, _, flow| {
		let mut pass = false;
		match event {
			Event::WindowEvent { window_id, event }
				if window_id == window.id() => {

				match event {
					WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
					WindowEvent::Resized(size) => {
						let PhysicalSize { width, height } = size;

						viewport.width = width;
						viewport.height = height;

						let aspect = f64::from(width) / f64::from(height);
						scene.aspect = aspect as f32;
					},
					WindowEvent::KeyboardInput { input, .. } => {
						let (button, state) = (input.scancode, input.state);

						match (button, state) {
							(57419, ElementState::Pressed)                      => direction = 1.0,
							(57419, ElementState::Released) if direction >= 0.0 => direction = 0.0,
							(57421, ElementState::Pressed)                      => direction = -1.0,
							(57421, ElementState::Released) if direction <= 0.0 => direction = 0.0,
							_ => {}
						}
					},
					_ => {}
				}
			},
			Event::MainEventsCleared => pass = true,
			_ => {},
		}
		if !pass { return }

		let delta = (delta_time)();
		if direction != 0.0 {
			angle += std::f32::consts::FRAC_PI_8 * delta.as_secs_f32() * direction.signum();
			angle.clamp(0.0, std::f32::consts::PI);
		}

		scene.light_position[0] = angle.cos() * 2.0;
		scene.light_position[1] = angle.sin() * 2.0;
		let _ = {
			let t = angle.sin();
			let t = t.clamp(0.0, 1.0);

			scene.light_color[0] = t * 0.486 + (1.0 - t) * 0.957;
			scene.light_color[1] = 0.792;
			scene.light_color[2] = t * 0.957 + (1.0 - t) * 0.486;
		};
		scene.update(delta);

		renderer.update(&scene);
		renderer.draw(&device, &framebuffer, viewport);

		(swap_buffers)();
	});
}



/* Generate the main function. */
environment::main!(run);