
use environment::Environment;
use winit::event_loop::ControlFlow;
use winit::event::{Event, WindowEvent, MouseButton, ElementState};
use gavle::*;
use std::time::Duration;
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

	let mut grow_direction = 0.0f32;

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
					WindowEvent::MouseInput { button, state, .. } => {
						match (button, state) {
							(MouseButton::Left, ElementState::Pressed)   => grow_direction += 1.0,
							(MouseButton::Left, ElementState::Released)  => grow_direction -= 1.0,
							(MouseButton::Right, ElementState::Pressed)  => grow_direction -= 1.0,
							(MouseButton::Right, ElementState::Released) => grow_direction += 1.0,
							_ => {}
						}
					},
					_ => {}
				}
			},
			Event::MainEventsCleared => pass = true,
			_ => {}
		}
		if !pass { return }

		/* Update the application. */
		let delta = delta_time();

		let grow_direction = if grow_direction == 0.0 {
			0.0
		} else {
			grow_direction.signum()
		};
		state.scale += 0.5 * delta.as_secs_f32() * grow_direction;

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
	/** Uniform 2D scale factor to be applied to the triangle model. */
	pub scale: f32,
}
impl ApplicationRenderState {
	/** Create a new application state structure with default parameters. */
	pub fn new() -> Self {
		Self {
			scale: 1.0
		}
	}
}

/** Uniform parameters passed on to the shader. */
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct ShaderParams {
	/** Model-World-View transformation matrix.
	 *
	 * This transformation maps a coordinate in model space into a coordinate
	 * in screen space. Normally, having one single matrix for mapping model
	 * space to screen space is incredibly wasteful. But, because we only really
	 * have one model to display, this is a fine compromise to make, for the
	 * sake of simplicity. */
	pub model_world_view: Matrix4
}

/** Structure responsible for rendering information in the example pass directly
 * into a target framebuffer, without any sort of processing. */
struct ApplicationRenderStateVisitor {
	/** The render pipeline used in the render pass. */
	pipeline: RenderPipeline,
	/** Vertex buffer containing data for the triangle model. */
	vertices: VertexBuffer,
	/** Index buffer containing data for the triangle model. */
	indices: IndexBuffer,
	/** Uniform data passed to the shaders in the render pass. */
	params: UniformBuffer,
	/** Uniform bind group passed on to the shader. */
	bind: UniformGroup
}
impl ApplicationRenderStateVisitor {
	/** Create a new instance of this render pass. */
	pub fn new(device: &Device) -> Self {
		const VERTICES: &'static [Vertex; 3] = &[
			Vertex::new_unchecked([-0.5, -0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked([ 0.5, -0.5, 0.0], [1.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked([ 0.0,  0.5, 0.0], [0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const INDICES: &'static [u16; 4] = &[0, 1, 2, 0];

		let vertices = device.create_vertex_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::bytes_of(VERTICES).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::bytes_of(VERTICES)).unwrap();
		let indices = device.create_index_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::bytes_of(INDICES).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::bytes_of(INDICES)).unwrap();

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
					topology: PrimitiveTopology::TriangleList,
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

		let params = device.create_uniform_buffer(
			&BufferDescriptor {
				size: u32::try_from(bytemuck::bytes_of(
					&ShaderParams::zeroed()).len()).unwrap(),
				profile: BufferProfile::DynamicUpload
			}).unwrap();
		let bind = device.create_uniform_bind_group(
			&UniformGroupDescriptor {
				entries: &[
					UniformGroupEntry {
						binding: "rc_params".into(),
						kind: UniformBind::Buffer {
							buffer: &params
						}
					}
				]
			});

		Self {
			pipeline,
			vertices,
			indices,
			params,
			bind
		}
	}

	/** Dispatch this render pass with the given parameters. */
	pub fn visit(
		&mut self,
		device: &Device,
		framebuffer: &Framebuffer,
		viewport: &Viewport,
		state: &ApplicationRenderState) {

		/* Upload the application state to the buffer holding parameter data. */
		let _ = {
			let params = ShaderParams {
				model_world_view: Matrix4::scale(
					state.scale,
					state.scale,
					state.scale)
			};

			let slice = self.params.slice(..);
			let mut map = slice.try_map_mut(BufferLoadOp::DontCare).unwrap();

			let data = bytemuck::bytes_of(&params);
			map[..data.len()].copy_from_slice(data);
		};

		/* Draw the triangle. */
		let mut pass = device.start_render_pass(
			&RenderPassDescriptor {
				pipeline: &self.pipeline,
				framebuffer
			});

		pass.set_bind_group(&self.bind);
		pass.set_index_buffer(&self.indices);
		pass.set_vertex_buffer(&self.vertices);
		pass.set_viewport(*viewport);

		pass.draw_indexed(
			0..5,
			1);
	}
}

/* Generate the main function. */
environment::main!(run);

