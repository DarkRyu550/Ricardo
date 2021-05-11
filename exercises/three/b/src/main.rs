
use environment::Environment;
use winit::event_loop::ControlFlow;
use winit::event::{Event, WindowEvent, ElementState, MouseButton, MouseScrollDelta};
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

	let mut dragging = false;
	let mut cursor_x = 0.0_f32;
	let mut cursor_y = 0.0_f32;

	/* Common parameters passed to the renderer. */
	let framebuffer = device.default_framebuffer(
		&DefaultFramebufferDescriptor {
			color_load_op: LoadOp::Clear(Color {
				red: 0.0,
				green: 0.0,
				blue: 0.0,
				alpha: 1.0
			}),
			depth_load_op: LoadOp::Clear(f32::INFINITY),
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
					WindowEvent::MouseInput { button, state, .. }
						if MouseButton::Left == button => {

						dragging = match state {
							ElementState::Pressed => true,
							ElementState::Released => false,
						}
					}
					WindowEvent::CursorMoved { position, .. } => {
						let x = (position.x / f64::from(viewport.width))  * 2.0 - 1.0;
						let y = (position.y / f64::from(viewport.height)) * 2.0 - 1.0;

						if dragging {
							let dx = cursor_x - x as f32;
							let dy = cursor_y - y as f32;

							state.yaw   -= dx * std::f32::consts::PI;
							state.pitch -= dy * std::f32::consts::PI;

							state.pitch = state.pitch.clamp(
								-std::f32::consts::FRAC_PI_2,
								 std::f32::consts::FRAC_PI_2);
						}

						cursor_x = x as f32;
						cursor_y = y as f32;
					},
					WindowEvent::MouseWheel { delta, .. } => {
						let delta = match delta {
							MouseScrollDelta::LineDelta(delta, _) => delta,
							MouseScrollDelta::PixelDelta(delta) =>
								((delta.y / f64::from(viewport.height))  * 2.0 - 1.0) as f32
						};

						state.distance += delta;
						state.distance = state.distance.clamp(2.0, 20.0)
					}
					_ => {}
				}
			},
			Event::MainEventsCleared => pass = true,
			_ => {}
		}
		if !pass { return }

		/* Update the application. */
		let _ = delta_time();

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
	/** Angle of yaw of the object. */
	pub yaw: f32,
	/** Angle of pitch of the object. */
	pub pitch: f32,
	/** Distance to the object. */
	pub distance: f32,
}
impl ApplicationRenderState {
	/** Create a new application state structure with default parameters. */
	pub fn new() -> Self {
		Self {
			yaw: 0.0,
			pitch: std::f32::consts::FRAC_PI_6,
			distance: 2.69
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
	bind: UniformGroup,
	/** Number of indices in the current model. */
	index_count: u32,
}
impl ApplicationRenderStateVisitor {
	/** Create a new instance of this render pass. */
	pub fn new(device: &Device) -> Self {
		let steps = 64_u16;
		let mut vertices = Vec::new();
		let mut indices = Vec::new();

		/* Generate the upper and lower discs. */
		for height in 0..=1 {
			let z = if height == 0 { -1.0 } else { 1.0 };
			let local_vertices = (0..steps)
				.into_iter()
				.map(|step| {
					let angle = 2.0 * std::f32::consts::PI / f32::from(steps - 1);
					let angle = angle * f32::from(step);

					let x = angle.cos();
					let y = angle.sin();

					Vertex::new_unchecked(
						[x, y, z],
						[x, y],
						[(x + 1.0) / 2.0, (y + 1.0) / 2.0, 1.0],
						[1.0, 0.0, 0.0],
						[0.0, 1.0, 0.0])
				});
			vertices.extend(local_vertices);

			let base = if height == 0 { 0 } else { steps };
			let local_indices = (base..base + steps)
				.skip(1)
				.zip((base..base + steps).skip(2))
				.flat_map(|(a, b)| {
					std::array::IntoIter::new([base, a, b])
				});
			indices.extend(local_indices);
		}

		/* Weave between them. */
		for triangle in 0..steps {
			/* Bottom up triangle. */
			let a0 = triangle;
			let b0 = (triangle + 1) % steps;
			let c0 = triangle + steps;

			/* Bottom down triangle. */
			let a1 = if triangle == 0 {
				triangle + steps + steps - 1
			} else {
				triangle + steps - 1
			};
			let b1 = triangle + steps;
			let c1 = triangle;

			indices.extend(std::array::IntoIter::new([a0, b0, c0]));
			indices.extend(std::array::IntoIter::new([a1, b1, c1]));
		}

		let index_count = indices.len() as u32;

		let vertices = device.create_vertex_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::cast_slice::<_, u8>(&vertices[..]).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::cast_slice(&vertices[..])).unwrap();
		let indices = device.create_index_buffer_with_data(
			&BufferDescriptor {
				size: bytemuck::cast_slice::<_, u8>(&indices[..]).len() as u32,
				profile: BufferProfile::StaticUpload
			},
			bytemuck::cast_slice(&indices[..])).unwrap();

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
				depth_stencil: Some(DepthStencilState {
					depth_write_enabled: true,
					depth_compare: CompareFunction::Less,
					stencil: StencilState::IGNORE
				})
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
					},
				]
			});

		Self {
			pipeline,
			vertices,
			indices,
			params,
			bind,
			index_count
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
				model_world_view: {
					let matrix = Matrix4::rotate(
						1.0,
						0.0,
						0.0,
						state.pitch);
					let matrix = Matrix4::rotate(
						0.0,
						1.0,
						0.0,
						state.yaw) * matrix;
					let matrix = Matrix4::translate(
						0.0,
						0.0,
						state.distance) * matrix;
					let matrix = Matrix4::rectilinear_projection(
						std::f32::consts::FRAC_PI_2,
						(f64::from(viewport.width) / f64::from(viewport.height)) as f32,
						1.0,
						100.0) * matrix;

					matrix.transpose()
				}
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
			0..self.index_count,
			1);
	}
}

/* Generate the main function. */
environment::main!(run);

