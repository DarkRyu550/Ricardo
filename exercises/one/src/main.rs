use environment::Environment;
use winit::event_loop::ControlFlow;
use winit::event::{Event, WindowEvent};
use gavle::*;
use std::collections::HashMap;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use std::borrow::Cow;

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

	/* Create the example render pass and some of the parameters we will be
	 * using throughout the loop. */
	let example_pass = ExamplePass::new(&device);

	let mut top_index = 0usize;
	let mut viewport = Viewport { x: 0, y: 0, width: 800, height: 600 };
	let mut clock = Duration::from_secs(0);

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
					}
					_ => {}
				}
			},
			Event::MainEventsCleared => pass = true,
			_ => {}
		}
		if !pass { return }

		/* Update the game. */
		clock += delta_time();
		while clock > Duration::from_secs(1) {
			clock -= Duration::from_secs(1);

			top_index += 1;
			top_index %= ExamplePass::TOPOLOGIES.len();
		}

		/* Render the application. */
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
		example_pass.dispatch(
			&device,
			&framebuffer,
			&viewport,
			ExamplePass::TOPOLOGIES[top_index]);

		swap_buffers();
	})
}

/** Structure responsible for rendering information in the example pass directly
 * into a target framebuffer, without any sort of processing. */
struct ExamplePass {
	/** All of the pipelines in this pass, sorted by the type of their topology. */
	pipelines: HashMap<PrimitiveTopology, RenderPipeline>,
	/** Vertex buffer containing data for the triangle model. */
	vertices: VertexBuffer,
	/** Index buffer containing data for the triangle model. */
	indices: IndexBuffer,
}
impl ExamplePass {
	/** List of the topologies supported by this render pass. */
	pub const TOPOLOGIES: &'static [PrimitiveTopology] = &[
		PrimitiveTopology::LineStrip,
		PrimitiveTopology::TriangleList,
	];

	/** Create a new instance of this render pass. */
	pub fn new(device: &Device) -> Self {
		const VERTICES: &'static [Vertex; 3] = &[
			Vertex { position: [-0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0], texture: [0.0, 1.0, 0.0] },
			Vertex { position: [ 0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0], texture: [1.0, 0.0, 0.0] },
			Vertex { position: [ 0.0,  0.5, 0.0], normal: [0.0, 0.0, 1.0], texture: [0.0, 0.0, 1.0] },
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
			assets::example::vertex()).unwrap();
		let fragment = device.create_fragment_shader(
			assets::example::fragment()).unwrap();

		let pipelines = Self::TOPOLOGIES.iter()
			.map(|topology| {
				let device = device.create_render_pipeline(
					&RenderPipelineDescriptor {
						vertex: VertexState {
							shader: &vertex,
							buffer: Vertex::LAYOUT
						},
						primitive_state: PrimitiveState {
							topology: *topology,
							index_format: IndexFormat::Uint16,
							front_face: FrontFace::Ccw,
							cull_mode: CullMode::None,
							polygon_mode: PolygonMode::Fill
						},
						fragment: Some(&fragment),
						depth_stencil: None
					}).unwrap();

				(*topology, device)
			}).collect();

		Self {
			pipelines,
			vertices,
			indices
		}
	}

	/** Dispatch this render pass with the given parameters. */
	pub fn dispatch(
		&self,
		device: &Device,
		framebuffer: &Framebuffer,
		viewport: &Viewport,
		topology: PrimitiveTopology) {

		let pipeline = match self.pipelines.get(&topology) {
			Some(pipeline) => pipeline,
			None =>
				panic!("tried to use invalid topology: {:?}. supported \
					topologies are {:?}",
					topology, Self::TOPOLOGIES)
		};

		let mut pass = device.start_render_pass(
			&RenderPassDescriptor { pipeline, framebuffer });

		pass.set_index_buffer(&self.indices);
		pass.set_vertex_buffer(&self.vertices);
		pass.set_viewport(*viewport);

		pass.draw_indexed(
			0..5,
			1);
	}
}

/** Vertex type used throughout this application. */
#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	/** Three-dimensional position data. */
	pub position: [f32; 3],
	/** Three-dimensional vertex normal data. */
	pub normal: [f32; 3],
	/** Three-dimensional UVW texture coordinates. */
	pub texture: [f32; 3]
}
impl Vertex {
	/** Layout data for buffers using this vertex type. */
	pub const LAYOUT: &'static VertexBufferLayout<'static> = &VertexBufferLayout {
		array_stride: 4 * 3 * 3,
		attributes: &[
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 0,
				binding: Cow::Borrowed("tt_vert_position")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 12,
				binding: Cow::Borrowed("tt_vert_normal")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 24,
				binding: Cow::Borrowed("tt_vert_texture")
			},
		]
	};
}

/* Generate the main function. */
environment::main!(run);
