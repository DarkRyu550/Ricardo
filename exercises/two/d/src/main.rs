
use environment::Environment;
use winit::event_loop::ControlFlow;
use winit::event::{Event, WindowEvent, ElementState};
use gavle::*;
use winit::dpi::PhysicalSize;
use support::{Vertex, Matrix4};
use std::convert::TryFrom;
use bytemuck::Zeroable;

/** Graphical assets used by this application. */
mod assets;

/** Function responsible for running the game inside of a given application
 * environment, provided by the [`environment`] crate. */
fn run(env: Environment) {
	let Environment {
		window,
		event_loop,
		device,
		mut swap_buffers,
		mut delta_time
	} = env;

	/* Initialize the application state and create the visitor that will be
	 * responsible for rendering the application state to the screen. */
	let mut state = ApplicationRenderState::new();
	let mut state_visitor = ApplicationRenderStateVisitor::new(&device);

	let mut direction_y = 0.0f32;
	let mut direction_x = 0.0f32;

	/* Common parameters passed to the renderer. */
	let framebuffer = device.default_framebuffer(
		&DefaultFramebufferDescriptor {
			color_load_op: LoadOp::Clear(Color {
				red: 0.0,
				green: 0.0,
				blue: 0.0,
				alpha: 1.0
			}),
			depth_load_op: LoadOp::Clear(f32::NEG_INFINITY),
			stencil_load_op: LoadOp::Clear(1)
		});
	let mut viewport = Viewport { x: 0, y: 0, width: 800, height: 600 };

	/* Run the main game loop. */
	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Poll;
		let mut pass = false;

		/* Process the events coming from the window. */
		match event {
			Event::WindowEvent { event, window_id }
			if window_id == window.id() => {
				match event {
					WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
					WindowEvent::Resized(size) => {
						let PhysicalSize { width, height } = size;
						viewport.width  = width;
						viewport.height = height;
					},
					WindowEvent::KeyboardInput { input, .. } => {
						let (button, state) = (input.scancode, input.state);

						match (button, state) {
							(17, ElementState::Pressed)                        => direction_y = 1.0,
							(17, ElementState::Released) if direction_y >= 0.0 => direction_y = 0.0,
							(30, ElementState::Pressed)                        => direction_x = -1.0,
							(30, ElementState::Released) if direction_x <= 0.0 => direction_x = 0.0,
							(31, ElementState::Pressed)                        => direction_y = -1.0,
							(31, ElementState::Released) if direction_y <= 0.0 => direction_y = 0.0,
							(32, ElementState::Pressed)                        => direction_x = 1.0,
							(32, ElementState::Released) if direction_x >= 0.0 => direction_x = 0.0,
							_ => {}
						}
					},
					WindowEvent::CursorMoved { position, .. } => {
						state.triangle_position[0] =  ((position.x / f64::from(viewport.width))  * 2.0 - 1.0) as f32;
						state.triangle_position[1] = -((position.y / f64::from(viewport.height)) * 2.0 - 1.0) as f32;
					}
					_ => {}
				}
			},
			Event::MainEventsCleared => pass = true,
			_ => {}
		}
		if !pass { return }

		/* Update the application. */
		let delta = delta_time();

		let direction_x = if direction_x == 0.0 {
			0.0
		} else {
			direction_x.signum()
		};
		let direction_y = if direction_y == 0.0 {
			0.0
		} else {
			direction_y.signum()
		};

		state.circle_position[0] += 0.5 * delta.as_secs_f32() * direction_x;
		state.circle_position[1] += 0.5 * delta.as_secs_f32() * direction_y;

		/* Render the application. */
		state_visitor.visit(
			&device,
			&framebuffer,
			&viewport,
			&state);

		swap_buffers();
	})
}

/** All of the data that makes up a given state of the application. */
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct ApplicationRenderState {
	/** Position of the center of the circle. */
	pub circle_position: [f32; 2],
	/** Position of the center of the triangle. */
	pub triangle_position: [f32; 2],
}
impl ApplicationRenderState {
	/** Create a new application state structure with default parameters. */
	pub fn new() -> Self {
		Self {
			circle_position: [0.0, 0.0],
			triangle_position: [0.0, 0.0],
		}
	}
}

/** Uniform parameters passed on to the shader. */
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct ShaderParams {
	/** Model-World-View transformation matrix to be applied to the circle.
	 *
	 * This transformation maps a coordinate in model space into a coordinate
	 * in screen space. Normally, having one single matrix for mapping model
	 * space to screen space is incredibly wasteful. But, because we only really
	 * have very few models to display, this is a fine compromise to make, for
	 * the sake of simplicity. */
	pub model_world_view: Matrix4,
}

/** Structure responsible for rendering information in the example pass directly
 * into a target framebuffer, without any sort of processing. */
struct ApplicationRenderStateVisitor {
	/** The render pipeline used in the render pass. */
	pipeline: RenderPipeline,

	/** Vertex buffer containing data for the triangle model. */
	circle_vertices: VertexBuffer,
	/** Index buffer containing data for the triangle model. */
	circle_indices: IndexBuffer,
	/** Number of indices in the current model. */
	circle_index_count: u32,

	/** Vertex buffer containing data for the triangle model. */
	triangle_vertices: VertexBuffer,
	/** Index buffer containing data for the triangle model. */
	triangle_indices: IndexBuffer,
	/** Number of indices in the current model. */
	triangle_index_count: u32,

	/** Uniform data passed to the shaders in the render pass. */
	circle_params: UniformBuffer,
	/** Uniform bind group passed on to the shader. */
	circle_bind: UniformGroup,

	/** Uniform data passed to the shaders in the render pass. */
	triangle_params: UniformBuffer,
	/** Uniform bind group passed on to the shader. */
	triangle_bind: UniformGroup,
}
impl ApplicationRenderStateVisitor {
	/** Create a new instance of this render pass. */
	pub fn new(device: &Device) -> Self {
		let steps = 64_u16;
		let circle_vertices = (0..steps)
			.into_iter()
			.map(|step| {
				let angle = 2.0 * std::f32::consts::PI / f32::from(steps - 1);
				let angle = angle * f32::from(step);

				let x = angle.cos();
				let y = angle.sin();

				Vertex::new_unchecked(
					[x, y, 0.0],
					[x, y],
					[(x + 1.0) / 2.0, (y + 1.0) / 2.0, 1.0],
					[1.0, 0.0, 0.0],
					[0.0, 1.0, 0.0])
			})
			.collect::<Vec<_>>();
		let circle_indices = (0..steps)
			.skip(1)
			.zip((0..steps).skip(2))
			.flat_map(|(a, b)| {
				std::array::IntoIter::new([0, a, b])
			})
			.collect::<Vec<_>>();
		let circle_index_count = circle_indices.len() as u32;

		let circle_vertices = device.create_vertex_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::cast_slice::<_, u8>(&circle_vertices[..]).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::cast_slice(&circle_vertices[..])).unwrap();
		let circle_indices = device.create_index_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::cast_slice::<_, u8>(&circle_indices[..]).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::cast_slice(&circle_indices[..])).unwrap();

		const TRIANGLE_VERTICES: &'static [Vertex; 3] = &[
			Vertex::new_unchecked([-0.5, -0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked([ 0.5, -0.5, 0.0], [1.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked([ 0.0,  0.5, 0.0], [0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const TRIANGLE_INDICES: &'static [u16; 4] = &[0, 1, 2, 0];

		let triangle_vertices = device.create_vertex_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::bytes_of(TRIANGLE_VERTICES).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::bytes_of(TRIANGLE_VERTICES)).unwrap();
		let triangle_indices = device.create_index_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::bytes_of(TRIANGLE_INDICES).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::bytes_of(TRIANGLE_INDICES)).unwrap();

		let vertex = device.create_vertex_shader(
			assets::visitor::vertex()).unwrap();
		let fragment = device.create_fragment_shader(
			assets::visitor::fragment()).unwrap();

		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				vertex: VertexState {
					shader: &vertex,
					buffer: &Vertex::LAYOUT
				},
				primitive_state: PrimitiveState {
					topology: PrimitiveTopology::TriangleStrip,
					index_format: IndexFormat::Uint16,
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::None,
					polygon_mode: PolygonMode::Fill
				},
				fragment: Some(FragmentState {
					shader: &fragment,
					targets: ColorTargetState {
						alpha_blend: BlendState::REPLACE,
						color_blend: BlendState::REPLACE,
						write_mask: ColorWrite::all(),
					}
				}),
				depth_stencil: None
			}).unwrap();

		let circle_params = device.create_uniform_buffer(
			&BufferDescriptor {
				size: u32::try_from(bytemuck::bytes_of(
					&ShaderParams::zeroed()).len()).unwrap(),
				profile: BufferProfile::DynamicUpload
			}).unwrap();
		let circle_bind = device.create_uniform_bind_group(
			&UniformGroupDescriptor {
				entries: &[
					UniformGroupEntry {
						binding: "rc_params".into(),
						kind: UniformBind::Buffer {
							buffer: &circle_params
						}
					}
				]
			});

		let triangle_params = device.create_uniform_buffer(
			&BufferDescriptor {
				size: u32::try_from(bytemuck::bytes_of(
					&ShaderParams::zeroed()).len()).unwrap(),
				profile: BufferProfile::DynamicUpload
			}).unwrap();
		let triangle_bind = device.create_uniform_bind_group(
			&UniformGroupDescriptor {
				entries: &[
					UniformGroupEntry {
						binding: "rc_params".into(),
						kind: UniformBind::Buffer {
							buffer: &triangle_params
						}
					}
				]
			});

		Self {
			pipeline,
			circle_vertices,
			circle_indices,
			circle_index_count,
			triangle_vertices,
			triangle_indices,
			triangle_index_count: TRIANGLE_INDICES.len() as u32,
			circle_params,
			circle_bind,
			triangle_params,
			triangle_bind
		}
	}

	/** Dispatch this render pass with the given parameters. */
	pub fn visit(
		&mut self,
		device: &Device,
		framebuffer: &Framebuffer,
		viewport: &Viewport,
		state: &ApplicationRenderState) {

		/* Upload the application state to the buffer holding parameter data for
		 * the circle. */
		let _ = {
			let params = ShaderParams {
				model_world_view: {
					let matrix = Matrix4::scale(0.2, 0.2, 1.0);
					let matrix = Matrix4::translate(
						state.circle_position[0],
						state.circle_position[1],
						0.0) * matrix;

					matrix.transpose()
				}
			};

			let slice = self.circle_params.slice(..);
			let mut map = slice.try_map_mut(BufferLoadOp::DontCare).unwrap();

			let data = bytemuck::bytes_of(&params);
			map[..data.len()].copy_from_slice(data);
		};

		/* Upload the application state to the buffer holding parameter data for
		 * the triangle. */
		let _ = {
			let params = ShaderParams {
				model_world_view: {
					let matrix = Matrix4::scale(0.2, 0.2, 1.0);
					let matrix = Matrix4::translate(
						state.triangle_position[0],
						state.triangle_position[1],
						0.0) * matrix;

					matrix.transpose()
				}
			};

			let slice = self.triangle_params.slice(..);
			let mut map = slice.try_map_mut(BufferLoadOp::DontCare).unwrap();

			let data = bytemuck::bytes_of(&params);
			map[..data.len()].copy_from_slice(data);
		};

		let mut pass = device.start_render_pass(
			&RenderPassDescriptor {
				pipeline: &self.pipeline,
				framebuffer
			});

		/* Draw the circle. */
		let _ = {
			pass.set_bind_group(&self.circle_bind);
			pass.set_index_buffer(&self.circle_indices);
			pass.set_vertex_buffer(&self.circle_vertices);
			pass.set_viewport(*viewport);

			pass.draw_indexed(
				0..self.circle_index_count,
				1);
		};

		/* Draw the triangle. */
		let _ = {
			pass.set_bind_group(&self.triangle_bind);
			pass.set_index_buffer(&self.triangle_indices);
			pass.set_vertex_buffer(&self.triangle_vertices);
			pass.set_viewport(*viewport);

			pass.draw_indexed(
				0..self.triangle_index_count,
				1);
		};
	}
}

/* Generate the main function. */
environment::main!(run);

