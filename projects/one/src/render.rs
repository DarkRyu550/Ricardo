use gavle::*;
use support::{Vertex, Matrix4, Camera, Projection};
use std::convert::TryFrom;
use crate::scene::Scene;
use std::hint::unreachable_unchecked;

pub struct Renderer {
	mountains: Mountains,
	snowfall: Snowfall,
	backwall: Backwall,
	waterfall: Waterfall,
	uniforms: Uniforms,
}
impl Renderer {
	pub fn new(device: &Device) -> Self {
		Self {
			mountains: Mountains::new(device),
			snowfall: Snowfall::new(device),
			backwall: Backwall::new(device),
			waterfall: Waterfall::new(device),
			uniforms: Uniforms::new(device),
		}
	}

	pub fn update(&mut self, scene: &Scene) {
		let mut iter = scene.snowflakes.entities.entities();
		self.uniforms.snowflakes
			.resize_with(
				scene.snowflakes.entities.len() as u32,
				|| {
					let snowflake = iter.next().unwrap();
					Instance::new(
						[
							snowflake.position[0],
							snowflake.position[1],
							1.2,
						],
						[1.0, 1.0])
				});
		self.uniforms.global
			.resize_with(
				1,
				|| Globals::new(
					scene.light_position,
					scene.light_color,
					[0.486, 0.792, 0.957],
					scene.camera,
					scene.aspect
				));
	}

	pub fn draw(&self, device: &Device, target: &Framebuffer, viewport: Viewport) {
		let mut pass = device.start_render_pass(
			&RenderPassDescriptor {
				pipeline: &self.snowfall.pipeline,
				framebuffer: target
			});

		pass.set_viewport(viewport);
		pass.set_stencil_reference(1);
		pass.set_bind_group(&self.uniforms.group);

		/* Render the snow. */
		pass.set_pipeline(&self.snowfall.pipeline);
		pass.set_vertex_buffer(&self.snowfall.geometry.0);
		pass.set_index_buffer(&self.snowfall.geometry.1);

		pass.draw_indexed(0..3, self.uniforms.snowflakes.len());

		/* Render the mountains. */
		pass.set_pipeline(&self.mountains.pipeline);
		pass.set_vertex_buffer(&self.mountains.geometry.0);
		pass.set_index_buffer(&self.mountains.geometry.1);

		pass.draw_indexed(0..27, self.uniforms.mountains.len());

		/* Render the backwall. */
		pass.set_pipeline(&self.backwall.pipeline);
		pass.set_vertex_buffer(&self.backwall.geometry.0);
		pass.set_index_buffer(&self.backwall.geometry.1);

		pass.draw_indexed(0..27, self.uniforms.backwalls.len());

		/* Render the waterfall. */
		pass.set_pipeline(&self.waterfall.pipeline);
		pass.set_vertex_buffer(&self.waterfall.geometry.0);
		pass.set_index_buffer(&self.waterfall.geometry.1);

		pass.draw_indexed(0..27, self.uniforms.waterfalls.len());
	}
}

pub struct Waterfall {
	pipeline: RenderPipeline,
	geometry: (VertexBuffer, IndexBuffer),
}

impl Waterfall {
	pub fn new(device: &Device) -> Self {
		const GEOMETRY: &'static [Vertex] = &[
			Vertex::new_unchecked_with_color([-0.05, -1.0, -0.1], [0.5, 1.0], [0.5, 0.5, 0.9], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.05,  1.0, -0.1], [0.5, 1.0], [0.5, 0.5, 0.9], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.05, -1.0, -0.1], [0.5, 1.0], [0.5, 0.5, 0.9], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.05,  1.0, -0.1], [0.5, 1.0], [0.5, 0.5, 0.9], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.035,  0.9, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.025,  0.8, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.025,  1.0, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.005,  0.6, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005,  0.5, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005,  0.7, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.005,  0.2, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005,  0.1, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005,  0.3, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.025, -0.2, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.035, -0.3, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.035, -0.1, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.035, -0.2, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.025, -0.3, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.025, -0.1, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.005, -0.6, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005, -0.7, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.005, -0.5, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.025, -0.9, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.035, -1.0, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.035, -0.8, -0.15], [0.5, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const INDICES: &'static [u16] = &[3, 1, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24];
		let geometry = upload_geometry(device, GEOMETRY, INDICES);

		use crate::shaders::waterfall as shaders;
		let vertex_shader = device.create_vertex_shader(shaders::VERTEX)
			.unwrap();
		let fragment_shader = device.create_fragment_shader(shaders::FRAGMENT)
			.unwrap();

		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				vertex: VertexState {
					shader: &vertex_shader,
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
					shader: &fragment_shader,
					targets: ColorTargetState {
						alpha_blend: BlendState::REPLACE,
						color_blend: BlendState::REPLACE,
						write_mask: ColorWrite::ALL
					}
				}),
				depth_stencil: Some(DepthStencilState {
					depth_write_enabled: true,
					depth_compare: CompareFunction::Less,
					stencil: StencilState::IGNORE
				})
			}).unwrap();

		Self { pipeline, geometry }
	}
}

pub struct Backwall {
	pipeline: RenderPipeline,
	geometry: (VertexBuffer, IndexBuffer),
}

impl Backwall {
	pub fn new(device: &Device) -> Self {
		const GEOMETRY: &'static [Vertex] = &[
			Vertex::new_unchecked_with_color([-1.0, -1.0, 0.0], [0.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-1.0,  1.0, 0.0], [0.5, 1.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0, -1.0, 0.0], [1.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0,  1.0, 0.0], [0.5, 1.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-1.0, -1.0, -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-1.0,  1.0, -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.8,  0.0, -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-1.0, -1.0, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.6, -1.0, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.8,  0.0, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.1,  1.0,  -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0,  0.35, -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.1,  1.0,  -0.01], [0.5, 1.0], [0.24, 0.276, 0.33], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.1,  1.0, -0.005], [0.5, 1.0], [0.16, 0.184, 0.22], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.2, -1.0, -0.005], [0.5, 1.0], [0.16, 0.184, 0.22], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0, -1.0, -0.005], [0.5, 1.0], [0.16, 0.184, 0.22], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.7,  1.0, -0.005], [0.5, 1.0], [0.16, 0.184, 0.22], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.2, -1.0,  -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.1, -0.35, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0, -1.0,  -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0, -1.0, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0,  0.0, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.9, -0.35, -0.01], [0.5, 1.0], [0.32, 0.368, 0.44], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const INDICES: &'static [u16] = &[3, 1, 0, 0, 2, 3, 6, 5, 4, 8, 9, 7, 12, 10, 11, 15, 13, 14, 15, 16, 13, 17, 18, 19, 20, 21, 22];
		let geometry = upload_geometry(device, GEOMETRY, INDICES);

		use crate::shaders::backwall as shaders;
		let vertex_shader = device.create_vertex_shader(shaders::VERTEX)
			.unwrap();
		let fragment_shader = device.create_fragment_shader(shaders::FRAGMENT)
			.unwrap();

		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				vertex: VertexState {
					shader: &vertex_shader,
					buffer: &Vertex::LAYOUT
				},
				primitive_state: PrimitiveState {
					topology: PrimitiveTopology::TriangleList,
					index_format: IndexFormat::Uint16,
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::Back,
					polygon_mode: PolygonMode::Fill
				},
				fragment: Some(FragmentState {
					shader: &fragment_shader,
					targets: ColorTargetState {
						alpha_blend: BlendState::REPLACE,
						color_blend: BlendState::REPLACE,
						write_mask: ColorWrite::ALL
					}
				}),
				depth_stencil: Some(DepthStencilState {
					depth_write_enabled: true,
					depth_compare: CompareFunction::Less,
					stencil: StencilState::IGNORE
				})
			}).unwrap();

		Self { pipeline, geometry }
	}
}

pub struct Snowfall {
	pipeline: RenderPipeline,
	geometry: (VertexBuffer, IndexBuffer),
}
impl Snowfall {
	pub fn new(device: &Device) -> Self {
		/* Specify the geometry of the particles and upload it. */
		const GEOMETRY: &'static [Vertex] = &[
			Vertex::new_unchecked_with_color([-1.0, -1.0, 0.0], [0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0, -1.0, 0.0], [1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0,  1.0, 0.0], [0.5, 1.0], [1.0, 1.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const INDICES: &'static [u16] = &[0, 1, 2];
		let geometry = upload_geometry(device, GEOMETRY, INDICES);

		use crate::shaders::snowfall as shaders;
		let vertex_shader = device.create_vertex_shader(shaders::VERTEX)
			.unwrap();
		let fragment_shader = device.create_fragment_shader(shaders::FRAGMENT)
			.unwrap();

		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				vertex: VertexState {
					shader: &vertex_shader,
					buffer: &Vertex::LAYOUT
				},
				primitive_state: PrimitiveState {
					topology: PrimitiveTopology::TriangleList,
					index_format: IndexFormat::Uint16,
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::Back,
					polygon_mode: PolygonMode::Fill
				},
				fragment: Some(FragmentState {
					shader: &fragment_shader,
					targets: ColorTargetState {
						alpha_blend: BlendState::REPLACE,
						color_blend: BlendState::REPLACE,
						write_mask: ColorWrite::ALL
					}
				}),
				depth_stencil: Some(DepthStencilState {
					depth_write_enabled: true,
					depth_compare: CompareFunction::Less,
					stencil: StencilState::IGNORE
				})
			}).unwrap();

		Self { pipeline, geometry }
	}
}

pub struct Mountains {
	pipeline: RenderPipeline,
	geometry: (VertexBuffer, IndexBuffer),
}
impl Mountains {
	const INSTANCES: u32 = 5;

	pub fn new(device: &Device) -> Self {
		/* Specify the geometry of the mountains in the background and upload them. */
		const GEOMETRY: &'static [Vertex] = &[
			Vertex::new_unchecked_with_color([-1.0, -1.0, 0.0], [0.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 1.0, -1.0, 0.0], [1.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0,  1.0, 0.0], [0.5, 1.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.2,  0.6, 0.0], [0.0, 0.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.2,  0.6, 0.0], [1.0, 0.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.1,  0.5, 0.0], [0.5, 1.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.1,  0.5, 0.0], [0.0, 0.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0,  0.6, 0.0], [0.5, 1.0], [0.90, 0.900, 0.95], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.2,  0.6, 0.0], [0.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.2,  0.6, 0.0], [1.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([-0.1,  0.5, 0.0], [0.5, 1.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.1,  0.5, 0.0], [0.0, 0.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
			Vertex::new_unchecked_with_color([ 0.0,  0.6, 0.0], [0.5, 1.0], [0.08, 0.092, 0.11], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
		];
		const INDICES: &'static [u16] = &[8, 0, 10, 9, 11, 1, 12, 0, 1, 3, 5, 7, 7, 6, 4, 2, 3, 7, 2, 7, 4, 10, 0, 12, 12, 1, 11];
		let geometry = upload_geometry(device, GEOMETRY, INDICES);

		use crate::shaders::mountains as shaders;
		let vertex_shader = device.create_vertex_shader(shaders::VERTEX)
				.unwrap();
		let fragment_shader = device.create_fragment_shader(shaders::FRAGMENT)
				.unwrap();

		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				vertex: VertexState {
					shader: &vertex_shader,
					buffer: &Vertex::LAYOUT
				},
				primitive_state: PrimitiveState {
					topology: PrimitiveTopology::TriangleList,
					index_format: IndexFormat::Uint16,
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::Back,
					polygon_mode: PolygonMode::Fill
				},
				fragment: Some(FragmentState {
					shader: &fragment_shader,
					targets: ColorTargetState {
						alpha_blend: BlendState::REPLACE,
						color_blend: BlendState::REPLACE,
						write_mask: ColorWrite::ALL
					}
				}),
				depth_stencil: Some(DepthStencilState {
					depth_write_enabled: true,
					depth_compare: CompareFunction::Less,
					stencil: StencilState {
						write_mask: 0xff,
						read_mask: 0xff,
						compare: CompareFunction::Always,
						fail_op: StencilOperation::Keep,
						depth_fail_op: StencilOperation::Keep,
						pass_op: StencilOperation::Replace
					}
				})
			}).unwrap();

		Self {
			pipeline,
			geometry,
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Instance {
	position: [f32; 3],
	_pad0: [u32; 1],
	scaling: [f32; 2],
	_pad1: [u32; 2],
}
impl Instance {
	pub fn new(position: [f32; 3], scaling: [f32; 2]) -> Self {
		Self {
			position,
			_pad0: [0; 1],
			scaling,
			_pad1: [0; 2]
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Globals {
	mountain_world: Matrix4,
	snowflake_world: Matrix4,
	backwall_world: Matrix4,
	waterfall_world: Matrix4,
	view_projection: Matrix4,

	light_position: [f32; 2],
	far_plane: f32,
	_pad0: [u32; 1],
	light_color: [f32; 3],
	_pad1: [u32; 1],
	transmission_tint: [f32; 3],
	_pad2: [u32; 1],
}
impl Globals {
	pub fn new(
		light_position: [f32; 2],
		light_color: [f32; 3],
		transmission_tint: [f32; 3],
		camera: Camera,
		aspect: f32) -> Self {

		let mountain_world = Matrix4::identity();
		let mountain_world = Matrix4::scale(
			0.5,
			0.5,
			1.0) * mountain_world;
		let mountain_world = mountain_world.transpose();

		let snowflake_world = Matrix4::identity();
		let snowflake_world = Matrix4::scale(
			0.005,
			0.005,
			1.0) * snowflake_world;
		let snowflake_world = snowflake_world.transpose();

		let backwall_world = Matrix4::identity();
		let backwall_world = Matrix4::scale(
			1.0,
			0.3,
			1.0) * backwall_world;
		let backwall_world = backwall_world.transpose();

		let waterfall_world = Matrix4::identity();
		let waterfall_world = Matrix4::scale(
			1.0,
			0.3,
			1.0) * waterfall_world;
		let waterfall_world = waterfall_world.transpose();

		let view_projection = camera.matrix(aspect);
		let view_projection = view_projection.transpose();

		let far_plane = match camera.projection {
			Projection::Perspective { far, .. } => far,
			Projection::Orthographic { far, .. } => far,
		};

		Self {
			mountain_world,
			snowflake_world,
			backwall_world,
			waterfall_world,
			view_projection,
			light_position,
			far_plane,
			_pad0: [0; 1],
			light_color,
			_pad1: [0; 1],
			transmission_tint,
			_pad2: [0; 1]
		}
	}
}

/** All of the uniform buffers used in this pass. */
struct Uniforms {
	global: UniformVec<Globals>,
	mountains: UniformVec<Instance>,
	snowflakes: UniformVec<Instance>,
	backwalls: UniformVec<Instance>,
	waterfalls: UniformVec<Instance>,

	group: UniformGroup,
}
impl Uniforms {
	const MAX_SNOWFLAKES: u32 = 4096;

	pub fn new(device: &Device) -> Self {
		let global = UniformVec::with_items(
			device,
			1,
			|| bytemuck::Zeroable::zeroed());

		let mut instance = 0u32;
		let mountains = UniformVec::with_items(
			device,
			5,
			|| {
				let data = Instance::new(
					match instance {
						0 => [-1.0, -0.1, 3.0],
						1 => [-0.5, -0.1, 2.0],
						2 => [ 0.0, -0.1, 3.0],
						3 => [ 0.5, -0.1, 2.0],
						4 => [ 1.0, -0.1, 3.0],
						_ => unreachable!()
					},
					[1.0, 1.0]);

				instance += 1;
				data
			});
		let snowflakes = UniformVec::with_capacity(
			device,
			Self::MAX_SNOWFLAKES);
		let backwalls = UniformVec::with_items(
			device,
			1,
			|| {
				Instance::new([0.0, -0.3, 1.0], [1.0, 1.0])
			}
		);
		let mut instance = 0u32;
		let waterfalls = UniformVec::with_items(
			device,
			2,
			|| {
				let data = Instance::new(
					match instance {
						0 => [ 0.45, -0.3, 1.0],
						1 => [-0.45, -0.3, 1.0],
						_ => unreachable!(),
					}, [1.0, 1.0]);
				instance += 1;
				data
			}
		);

		let group = device.create_uniform_bind_group(
			&UniformGroupDescriptor {
				entries: &[
					UniformGroupEntry {
						binding: "rc_global".into(),
						kind: UniformBind::Buffer {
							buffer: global.buffer()
						}
					},
					UniformGroupEntry {
						binding: "rc_mountains".into(),
						kind: UniformBind::Buffer {
							buffer: mountains.buffer()
						}
					},
					UniformGroupEntry {
						binding: "rc_snowflakes".into(),
						kind: UniformBind::Buffer {
							buffer: snowflakes.buffer()
						}
					},
					UniformGroupEntry {
						binding: "rc_backwalls".into(),
						kind: UniformBind::Buffer {
							buffer: backwalls.buffer(),
						}
					},
					UniformGroupEntry {
						binding: "rc_waterfalls".into(),
						kind: UniformBind::Buffer {
							buffer: waterfalls.buffer(),
						}
					}
				]
			});

		Self {
			global,
			mountains,
			snowflakes,
			backwalls,
			waterfalls,
			group
		}
	}
}

/** Vector of a given type in [`UniformBuffer`]-backed storage. */
struct UniformVec<T> {
	buffer: UniformBuffer,
	item_size: u32,
	max_items: u32,
	items: u32,
	_param: std::marker::PhantomData<T>,
}
impl<T> UniformVec<T>
	where T: bytemuck::Pod {

	pub fn with_capacity(device: &Device, capacity: u32) -> Self {
		let item: T = bytemuck::Zeroable::zeroed();
		let item_size = u32::try_from(bytemuck::bytes_of(&item).len())
			.expect("The size of one element in this buffer does not fit \
				into an unsigned 32-bit integer.");

		let max_items = device.information()
			.limits
			.max_uniform_block_size / item_size;
		let max_items = max_items.min(capacity);

		let buffer = device.create_uniform_buffer(
			&BufferDescriptor {
				size: max_items * item_size,
				profile: BufferProfile::DynamicUpload
			}).unwrap();

		Self {
			buffer,
			item_size,
			max_items,
			items: 0,
			_param: Default::default()
		}
	}

	pub fn with_items(
		device: &Device,
		items: u32,
		f: impl FnMut() -> T) -> Self {

		let mut this = Self::with_capacity(device, items);
		this.resize_with(items, f);

		this
	}

	/** Repopulates the data in the buffer with the given generator function. */
	pub fn resize_with(
		&mut self,
		items: u32,
		mut f: impl FnMut() -> T) {

		let items = if items > self.max_items {
			log::warn!("Clipping the number of populated items in the buffer \
				from the requested {} items to the maximum of {} items",
				items, self.max_items);
			self.max_items
		} else {
			items
		};

		let size = self.item_size * items;

		let slice = self.buffer.slice(..size);
		let mut map = slice.try_map_mut(BufferLoadOp::DontCare)
			.unwrap();

		let mut offset = 0;
		for _ in 0..items {
			(&mut map[offset as usize..(offset + self.item_size) as usize])
				.copy_from_slice(bytemuck::bytes_of(&(f)()));
			offset += self.item_size;
		}

		self.items = items;
	}

	/** The maximum capacity of this buffer, in elements. */
	pub fn capacity(&self) -> u32 {
		self.max_items
	}

	/** The number of items in this buffer. */
	pub fn len(&self) -> u32 {
		self.items
	}

	pub fn buffer(&self) -> &UniformBuffer {
		&self.buffer
	}
}

/** Uploads geometry to the device. */
fn upload_geometry(device: &Device, vertices: &[Vertex], indices: &[u16])
	-> (VertexBuffer, IndexBuffer) {
	let vert_size = {
		let vert: Vertex = bytemuck::Zeroable::zeroed();
		let size = bytemuck::bytes_of(&vert);

		u32::try_from(size.len())
			.expect("The size of a vertex cannot be converted into an \
					unsigned 32-bit integer.")
	};

	let vertices = device.create_vertex_buffer_with_data(
		&BufferDescriptor {
			size: {
				let count = u32::try_from(vertices.len())
					.expect("The number of vertices to be uploaded \
							does not fit into an unsigned 32-bit integer.");
				let size = vert_size.checked_mul(count)
					.expect("The number of bytes that would be taken up by \
							the total number of vertices does not fit into an \
							unsigned 32-bit integer.");

				size
			},
			profile: BufferProfile::StaticUpload
		},
		bytemuck::cast_slice(vertices))
		.expect("Could not upload vertex buffer data.");
	let indices = device.create_index_buffer_with_data(
		&BufferDescriptor {
			size: {
				let one = u32::try_from(std::mem::size_of::<i16>())
					.expect("The size of an u16 in bytes does not fit \
							into an u32 value. What kind of architecture are \
							you even using!?");
				let count = u32::try_from(indices.len())
					.expect("The number of indices to be uploaded \
							does not fit into an unsigned 32-bit integer.");
				let size = one.checked_mul(count)
					.expect("The number of bytes that would be taken up by \
							the total number of indices does not fit into an \
							unsigned 32-bit integer.");

				size
			},
			profile: BufferProfile::StaticUpload,
		},
		bytemuck::cast_slice(indices))
		.expect("Could not upload index buffer data.");

	(vertices, indices)
}

