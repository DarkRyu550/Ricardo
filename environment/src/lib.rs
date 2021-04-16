use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::window::{WindowBuilder, Window};
use winit::event_loop::EventLoop;
use gavle::Device;

/** Structures generated from the environment the application is running in. */
pub struct Environment {
	/** The window that was created for this application. */
	pub window: Window,
	/** The event loop attached to the window. */
	pub event_loop: EventLoop<()>,
	/** The device used to render the game. */
	pub device: Device,
	/** A function used to swap buffers in the display device. */
	pub swap_buffers: Box<dyn FnMut()>,
	/** A function used to gather the time since since the last call to itself. */
	pub delta_time: Box<dyn FnMut() -> Duration>
}

/**
 This macro generates the main functions for a given system, which then call
 the function given to this macro as a parameter to take up the responsibility
 of running the game or demo.

 An example of how this macro should be used, calling a function named `run` is
 as follows:

 ```rust,norun
 /** Function containing your incredible game or demo code. */
 fn run(_: environment::Environment) {}

 /* This call generates all the necessary scaffolding to create a main function
  * which starts up the rendering library, sets up the window and then hands off
  * control to the run function. */
 environment::main!(run);
 ```
 */
#[macro_export]
macro_rules! main {
	($main:ident) => {
		#[cfg(target_arch = "wasm32")]
		#[wasm_bindgen::prelude::wasm_bindgen(start)]
		pub fn wasm_start() {
			main()
		}

		fn main() {
			use environment::inner_start;
			let env = inner_start();
			$main(env);
		}
	}
}

/** Creates a new window and event loop pair. */
fn window() -> (EventLoop<()>, WindowBuilder) {
	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::default()
		.with_title("Ricardo")
		.with_resizable(true)
		.with_inner_size(PhysicalSize {
			width: 800,
			height: 600
		});

	(event_loop, window)
}

/** Inner part of the start function. Clients should use [the main! macro]
 * instead of this function in pretty much every case. */
#[cfg(not(target_arch = "wasm32"))]
pub fn inner_start() -> Environment {
	env_logger::init();
	let (event_loop, window_builder) = window();

	let windowed_context = glutin::ContextBuilder::new()
		.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)))
		.with_gl_profile(glutin::GlProfile::Core)
		.with_vsync(false)
		.with_multisampling(8)
		.build_windowed(window_builder, &event_loop)
		.expect("could not initialize opengl context");

	let context = match unsafe { windowed_context.make_current() } {
		Ok(context) => context,
		Err((_, what)) =>
			panic!("could not use the created opengl context: {}", what)
	};

	let device = gavle::Device::new_from_context(unsafe {
		glow::Context::from_loader_function(|proc| {
			context.get_proc_address(proc) as *const _
		})
	}).unwrap();

	let (context, window) = unsafe { context.split() };

	use std::time::Instant;
	let mut now = Instant::now();
	let mut frames = 0u32;
	let mut dnow = Instant::now();

	let environment = Environment {
		window,
		event_loop,
		device,
		swap_buffers: Box::new(move || context.swap_buffers().unwrap()),
		delta_time: Box::new(move || {
			let ndnow = Instant::now();
			let delta = ndnow.duration_since(dnow);
			dnow = ndnow;

			frames += 1;
			let elapsed = now.elapsed();
			if elapsed >= Duration::from_secs(1) {
				let fps = f64::from(frames) / elapsed.as_secs_f64();
				log::info!("FPS: {:.02}", fps);

				now = Instant::now();
				frames = 0;
			}

			delta
		})
	};
	environment
}

/** Inner part of the start function. Clients should use [the main! macro]
 * instead of this function in pretty much every case. */
#[cfg(target_arch = "wasm32")]
pub fn inner_start() -> Environment {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));

	console_log::init_with_level(log::Level::Trace)
		.expect("could not initialize logger");

	let (event_loop, window_builder) = window();
	let window = window_builder.build(&event_loop)
		.expect("could not create window");

	let canvas = winit::platform::web::WindowExtWebSys::canvas(&window);
	web_sys::window()
		.expect("no window element")
		.document()
		.expect("no document element")
		.body()
		.expect("document has no body")
		.append_child(&canvas)
		.expect("could not append canvas to body");

	use wasm_bindgen::JsCast;
	let context = canvas.get_context("webgl2")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::WebGl2RenderingContext>()
		.unwrap();
	let context = glow::Context::from_webgl2_context(context);

	let environment = Environment {
		window,
		event_loop,
		device: Device::new_from_context(context).unwrap(),
		swap_buffers: Box::new(move || {}),
		delta_time: Box::new(move || Duration::from_secs_f64(0.01666666666))
	};
	environment
}