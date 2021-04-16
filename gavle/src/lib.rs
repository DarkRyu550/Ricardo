#[macro_use]
extern crate log;

use glow::{HasContext, Context};
use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryFrom;
use crate::texture::InnerTexture;

mod buffer;
mod pipeline;
mod shader;
mod pass;
mod binding;
mod texture;
mod access;
mod framebuffer;
mod info;

pub use buffer::*;
pub use pipeline::*;
pub use shader::*;
pub use pass::*;
pub use binding::*;
pub use texture::*;
pub use framebuffer::*;
pub use info::*;

/** This macro instances shader creation functions from a common base. */
macro_rules! instance_shader_creation_functions {
	($(
		$(#[$outer:meta])*
		pub fn $name:ident: $shader:ident;
	)+) => {$(
		$(#[$outer])*
		pub fn $name(&self, source: ShaderSource)
			-> Result<$shader, ShaderError> {

			let gl = self.context.as_ref();
			let shader = unsafe {
				let shader = gl.create_shader(<$shader>::GL_TYPE)
					.map_err(|what| ShaderError::CreationFailed { what })?;

				match source {
					ShaderSource::Glsl(source) =>
						gl.shader_source(shader, &source)
				}

				gl.compile_shader(shader);
				if !gl.get_shader_compile_status(shader) {
					let what = gl.get_shader_info_log(shader);
					return Err(ShaderError::CompilationFailed { what })
				}

				shader
			};

			Ok($shader {
				inner: Rc::new(InnerShader {
					context: self.context.clone(),
					access: Default::default(),
					shader
				}),
			})
		}
	)+}
}
/** This macro instances buffer creation functions from a common base. */
macro_rules! instance_initialized_buffer_creation_functions {
	($(
		$(#[$outer:meta])*
		pub fn $name:ident: $buffer:ident;
	)+) => {$(
		$(#[$outer])*
		pub fn $name<A: AsRef<[u8]>>(
			&self,
			descriptor: &BufferDescriptor,
			data_: A)
			-> Result<$buffer, BufferError> {

			let	data = data_.as_ref();

			let len = u32::try_from(data.len());
			let len = match len {
				Ok(len) if len != descriptor.size =>
					panic!("the desired length of the uniform buffer ({}) and the \
						size of the initialization buffer ({}) must have been the \
						same", descriptor.size, len),
				Ok(len) => len,
				Err(what) =>
					panic!("the length of the initialization buffer does not fit \
						in a u32 value, as is required by opengl: {}", what),
			};

			let gl = self.context.as_ref();
			let buffer = unsafe {
				let buffer = gl.create_buffer()
					.map_err(|what| BufferError::CreationFailed { what })?;

				gl.bind_buffer(<$buffer>::GL_BIND, Some(buffer));
				gl.buffer_data_u8_slice(
					<$buffer>::GL_BIND,
					data,
					descriptor.profile.as_opengl());
				gl.bind_buffer(<$buffer>::GL_BIND, None);

				buffer
			};

			Ok($buffer {
				inner: Rc::new(InnerBuffer {
					context: self.context.clone(),
					information: self.information.clone(),
					pipeline: self.pipeline_lock.clone(),
					buffer,
					access: Default::default(),
					map: Default::default(),
					len
				})
			})
		}
	)+}
}
/** This macro instances buffer creation functions from a common base. */
macro_rules! instance_zero_initialized_buffer_creation_functions {
	($(
		$(#[$outer:meta])*
		pub fn $name:ident: $base:ident -> $buffer:ident;
	)+) => {$(
		$(#[$outer])*
		pub fn $name(
			&self,
			descriptor: &BufferDescriptor)
			-> Result<$buffer, BufferError> {

			let len  = usize::try_from(descriptor.size).unwrap();
			let init = vec![0; len];

			self.$base(descriptor, &init[..])
		}
	)+}
}

pub struct Device {
	/** Inner OpenGL context. */
	context: Rc<Context>,
	/** Information on the context. */
	information: Rc<Information>,
	/** Shared pipeline lock.
	 *
	 * Because of the way the pipeline is managed through an internal state
	 * machine in OpenGL, in order to avoid state corruption, we have to treat
	 * drawing commands as atomic transactions.
	 *
	 * This structure helps us support that behavior. */
	pipeline_lock: Rc<RefCell<()>>,
}
impl Device {
	/** Creates a new device from the given context, obtained externally to the
	 * device itself. This is useful in contexts in which the device does not
	 * or would not know how to properly create a context from scratch. */
	pub fn new_from_context(context: Context) -> Result<Self, UnsupportedContext> {
		let information = Information::collect(&context)?;
		debug!("Collected information: {:#?}", information);

		let context = Rc::new(context);
		Ok(Self {
			pipeline_lock: Rc::new(RefCell::new(())),
			information: Rc::new(information),
			context,
		})
	}

	/** Information on the current context. */
	pub fn information(&self) -> &Information {
		&*self.information
	}

	/** Creates a new uniform bind group from the given description. */
	pub fn create_uniform_bind_group(
		&self,
		description: &UniformGroupDescriptor)
		-> UniformGroup {

		let mut buffers = 0u32;
		let mut textures = 0u32;

		let mut entries = Vec::with_capacity(description.entries.len());
		for entry in description.entries {
			let bind = entry.binding.to_string();
			let kind = match entry.kind {
				UniformBind::Texture { texture, far, near } => {
					textures += 1;

					OwnedUniformBind::Texture {
						texture: Texture { inner: texture.inner.clone() },
						far,
						near
					}
				},
				UniformBind::Buffer { buffer } => {
					buffers += 1;

					if buffer.len() > self.information
						.limits
						.max_uniform_block_size {

						panic!("tried to use a uniform buffer larger than the \
							maximum size allowed for a single uniform binding: \
							len = {} > max = {}",
							buffer.len(),
							self.information
								.limits
								.max_uniform_block_size)
					}

					OwnedUniformBind::Buffer {
						buffer: UniformBuffer { inner: buffer.inner.clone() }
					}
				},
			};

			/* Make sure we haven't used bound resources than is allowed. */
			if buffers > self.information.limits.max_uniform_block_bindings {
				panic!("tried to use more uniform buffer bindings than is \
					allowed by the implementation. the maximum number of \
					uniform buffer bindings is {}",
					self.information.limits.max_uniform_block_bindings)
			}
			if textures > self.information.limits.max_textures {
				panic!("tried to use more texture bindings than is allowed by \
					the implementation. the maximum number of texture bindings \
					is {}",
					self.information.limits.max_textures)
			}

			entries.push((bind, kind));
		}

		UniformGroup {
			entries: Rc::new(entries)
		}
	}

	/** Get a handle to the default framebuffer, used to render to the screen
	 * and completely managed by OpenGL. */
	pub fn default_framebuffer(&self,
		descriptor: &DefaultFramebufferDescriptor) -> Framebuffer {

		Framebuffer {
			variants: FramebufferVariants::Default {
				color_load_op: descriptor.color_load_op,
				depth_load_op: descriptor.depth_load_op,
				stencil_load_op: descriptor.stencil_load_op
			}
		}
	}

	/** Tries to create a new framebuffer. Keep in mind that framebuffers
	 * created with this function can only be used for off-screen rendering.
	 *
	 * If you wish to render to the screen, instead, use the result from the
	 * [`default_framebuffer()`] function. */
	pub fn create_framebuffer(
		&self,
		descriptor: &FramebufferDescriptor)
		-> Result<Framebuffer, FramebufferError> {

		let _atom = self.pipeline_lock.borrow_mut();

		/* This function checks the extents of an attachment if that kind of
		 * information is available to us. */
		let check_extent = |width, height| {
			let max_attachment_width = self.information
				.limits
				.max_framebuffer_attachment_width;
			let max_attachment_height = self.information
				.limits
				.max_framebuffer_attachment_height;

			let extent = (
				max_attachment_width,
				max_attachment_height);
			if let (Some(max_width), Some(max_height)) = extent {
				if width > max_width {
					panic!("cannot use texture with width of {} as a \
							framebuffer attachment. the maximum width allowed \
							for framebuffer attachments is {}",
						width,
						max_width)
				}
				if height > max_height {
					panic!("cannot use texture with height of {} as a \
							framebuffer attachment. the maximum height allowed \
							for framebuffer attachments is {}",
						height,
						max_height)
				}
			}
		};

		let gl = self.context.as_ref();
		let framebuffer = unsafe {
			let framebuffer = gl.create_framebuffer()
				.map_err(|what| FramebufferError::CreationError { what })?;

			gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
			let bind_texture = |
				texture: &Texture,
				attachment: u32| match texture.inner.extent {
				TextureExtent::D1 { .. } | TextureExtent::D3 { .. } =>
					panic!("cannot bind a one-dimensional or three-dimensional \
						texture to a framebuffer"),
				TextureExtent::D2 { width, height } => {
					check_extent(width, height);

					gl.framebuffer_texture_2d(
						glow::FRAMEBUFFER,
						attachment,
						glow::TEXTURE_2D,
						Some(texture.inner.texture),
						0)
				},
				TextureExtent::D2Array { width, height, .. } => {
					warn!("using the first layer of the array texture for the \
						framebuffer attachment");
					check_extent(width, height);

					gl.framebuffer_texture_layer(
						glow::FRAMEBUFFER,
						attachment,
						Some(texture.inner.texture),
						0,
						0)
				}
			};

			let attachments = (0u32..).zip(descriptor.color_attachments);
			for (i, texture) in attachments {
				if i >= self.information
					.limits
					.max_framebuffer_color_attachments {

					panic!("the total number of color attachments would be \
						more than the maximum number of allowed attachments");
				}

				let attachment = glow::COLOR_ATTACHMENT0 + i;
				bind_texture(texture.attachment, attachment);
			}

			let attachments = &descriptor.depth_stencil_attachment;
			for texture in attachments {
				match texture.attachment.format() {
					TextureFormat::Depth24Stencil8 => {},
					_ => panic!("tried to bind to the depth-stencil attachment \
						a texture whose format is not a depth-stencil format: \
						{:?}", texture.attachment.format())
				}
				bind_texture(texture.attachment, glow::DEPTH_STENCIL_ATTACHMENT);
			}

			match gl.check_framebuffer_status(glow::FRAMEBUFFER) {
				glow::FRAMEBUFFER_COMPLETE => { /* Okay. */ },
				glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT =>
					panic!("the given attachments are framebuffer incomplete"),
				glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT =>
					panic!("no attachments were given to the framebuffer"),
				other =>
					panic!("framebuffer creation error: 0x{:08x}", other)
			}
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);

			framebuffer
		};

		Ok(Framebuffer {
			variants: FramebufferVariants::Custom {
				inner: Rc::new(InnerFramebuffer {
					context: self.context.clone(),
					access: Default::default(),
					color_attachments: Default::default(),
					depth_stencil: Default::default(),
					framebuffer,
					color_load_op: descriptor.color_attachments.get(0)
						.map(|attachment| attachment.load_op)
						.unwrap_or(LoadOp::Load),
					depth_load_op: descriptor.depth_stencil_attachment
						.map(|attachment| attachment.depth_load_op)
						.unwrap_or(LoadOp::Clear(f32::INFINITY)),
					stencil_load_op: descriptor.depth_stencil_attachment
						.map(|attachment| attachment.stencil_load_op)
						.unwrap_or(LoadOp::Clear(0xff)),
				})
			}
		})
	}

	/** Lock the render pipeline and start a new render pass from the given
	 * parameters. */
	pub fn start_render_pass<'a>(
		&'a self,
		descriptor: &RenderPassDescriptor<'a>)
		-> RenderPass<'a> {

		RenderPass {
			context: self.context.clone(),
			information: self.information.clone(),
			_lock: self.pipeline_lock.borrow_mut(),
			general_setup: false,
			pipeline: descriptor.pipeline,
			vertex: None,
			index: None,
			bind: None,
			framebuffer: descriptor.framebuffer,
			stencil_reference: 0,
			stencil_setup: false,
			draw_buffers_setup: false
		}
	}

	/** Internal implementation of the texture creation function, supporting
	 * creation of both user-initialized textures and default-initialized ones. */
	fn create_texture_generic(
		&self,
		descriptor: &TextureDescriptor,
		data: Option<&[u8]>)
		-> Result<Texture, TextureError> {

		let _atom = self.pipeline_lock.borrow_mut();

		/* Party rockers in the house tonight. */
		match descriptor.mip {
			Mipmap::None => {},
			Mipmap::Manual { .. } =>
				panic!("Mipmaps aren't supported yet sorry bro."),
			Mipmap::Automatic =>
				panic!("Mipmaps aren't supported yet sorry bro."),
		}

		let gl = self.context.as_ref();
		let texture = unsafe {
			let texture = gl.create_texture()
				.map_err(|what| TextureError::CreationError {what})?;

			let (format, internal_format, kind) = match descriptor.format {
				TextureFormat::Rgba8Unorm => (
					glow::RGBA,
					glow::RGBA8,
					glow::UNSIGNED_BYTE),
				TextureFormat::Rgba32Float => (
					glow::RGBA,
					glow::RGBA32F,
					glow::FLOAT),
				TextureFormat::Depth24Stencil8 => (
					glow::DEPTH_STENCIL,
					glow::DEPTH24_STENCIL8,
					glow::UNSIGNED_INT_24_8)
			};

			/* Check the the requested texture size against the limits imposed
			 * by the implementation. */
			{
				let (
					max_width,
					max_height,
					max_depth
				) = match descriptor.extent {
					TextureExtent::D1 { .. } => (
						self.information.limits.max_texture_size,
						1,
						1),
					TextureExtent::D2 { .. } => (
						self.information.limits.max_texture_size,
						self.information.limits.max_texture_size,
						1),
					TextureExtent::D2Array { .. } => (
						self.information.limits.max_texture_size,
						self.information.limits.max_texture_size,
						self.information.limits.max_texture_layers,
					),
					TextureExtent::D3 { .. } => (
						self.information.limits.max_texture_size_3d,
						self.information.limits.max_texture_size_3d,
						self.information.limits.max_texture_size_3d)
				};

				let (width, height, depth) = match descriptor.extent {
					TextureExtent::D1 { length } => (length, 1, 1),
					TextureExtent::D2 { width, height } => (width, height, 1),
					TextureExtent::D2Array { width, height, layers } =>
						(width, height, layers),
					TextureExtent::D3 { width, height, depth } =>
						(width, height, depth)
				};

				if width > max_width {
					panic!("tried to created texture with width ({}) greater \
						than the maximum width allowed by the implementation \
						({})",
						width,
						max_width)
				}
				if height > max_height {
					panic!("tried to created texture with height ({}) greater \
						than the maximum height allowed by the implementation \
						({})",
						height,
						max_height)
				}
				if depth > max_depth {
					panic!("tried to created texture with depth ({}) greater \
						than the maximum depth allowed by the implementation \
						({})",
						depth,
						max_depth)
				}
			}

			/* Check the size of the initialization data, if there is any. */
			if let Some(data) = data {
				let (columns, rows, pages) = match descriptor.extent {
					TextureExtent::D1 { length } => (length, 1, 1),
					TextureExtent::D2 { width, height } => (width, height, 1),
					TextureExtent::D2Array { width, height, layers } =>
						(width, height, layers),
					TextureExtent::D3 { width, height, depth } =>
						(width, height, depth)
				};
				let mips = match descriptor.mip {
					Mipmap::Automatic | Mipmap::None => 1,
					Mipmap::Manual { levels } => levels.get()
				};

				let bytes_per_pixel = match descriptor.format {
					TextureFormat::Rgba32Float => 4 * 4,
					TextureFormat::Rgba8Unorm  => 4 * 1,
					TextureFormat::Depth24Stencil8 => 1 * 4
				};

				let bytes_per_row = bytes_per_pixel * columns;
				let bytes_per_page = bytes_per_row * rows;
				let bytes_per_bundle = bytes_per_page * mips;
				let len = bytes_per_bundle * pages;

				if data.len() < usize::try_from(len).unwrap() {
					panic!("length of the intialization buffer ({}) is less \
						than the minimum required length for the texture that \
						would be created ({})",
						data.len(),
						len);
				}
			}

			/* Check whether a value is valid for the OpenGL FFI. */
			let check_i32 = |val: u32|
				i32::try_from(val).map_err(|what| TextureError::InvalidBounds {
					what: format!("the bounds must have fit in an i32: {:?}", what)
				});

			match descriptor.extent {
				TextureExtent::D1 { length } => {
					let length = check_i32(length)?;

					gl.bind_texture(glow::TEXTURE_1D, Some(texture));
					gl.tex_image_1d(
						glow::TEXTURE_1D,
						0,
						i32::try_from(internal_format).unwrap(),
						length,
						0,
						format,
						kind,
						data)
				},
				TextureExtent::D2 { width, height } => {
					let width = check_i32(width)?;
					let height = check_i32(height)?;

					gl.bind_texture(glow::TEXTURE_2D, Some(texture));
					gl.tex_image_2d(
						glow::TEXTURE_2D,
						0,
						i32::try_from(internal_format).unwrap(),
						width,
						height,
						0,
						format,
						kind,
						data);
				},
				TextureExtent::D2Array { width, height, layers } => {
					let width = check_i32(width)?;
					let height = check_i32(height)?;
					let layers = check_i32(layers)?;

					gl.bind_texture(glow::TEXTURE_3D, Some(texture));
					gl.tex_image_3d(
						glow::TEXTURE_3D,
						0,
						i32::try_from(internal_format).unwrap(),
						width,
						height,
						layers,
						0,
						format,
						kind,
						data);
				},
				TextureExtent::D3 { width, height, depth } => {
					let width = check_i32(width)?;
					let height = check_i32(height)?;
					let depth = check_i32(depth)?;

					gl.bind_texture(glow::TEXTURE_3D, Some(texture));
					gl.tex_image_3d(
						glow::TEXTURE_3D,
						0,
						i32::try_from(internal_format).unwrap(),
						width,
						height,
						depth,
						0,
						format,
						kind,
						data);
				}
			}

			texture
		};

		Ok(Texture {
			inner: Rc::new(InnerTexture {
				context: self.context.clone(),
				texture,
				access: Default::default(),
				format: descriptor.format,
				extent: descriptor.extent
			})
		})
	}

	/** Create a new texture from the given data. */
	pub fn create_texture_with_data<A: AsRef<[u8]>>(
		&self,
		descriptor: &TextureDescriptor,
		data_: A)
		-> Result<Texture, TextureError> {

		let data = data_.as_ref();
		self.create_texture_generic(
			descriptor,
			Some(data))
	}

	/** Create a new, default initialized texture. */
	pub fn create_texture(
		&self,
		descriptor: &TextureDescriptor)
		-> Result<Texture, TextureError> {

		self.create_texture_generic(
			descriptor,
			None)
	}

	/** Tries to create a new render pipeline from the given description. */
	pub fn create_render_pipeline(
		&self,
		descriptor: &RenderPipelineDescriptor)
		-> Result<RenderPipeline, RenderPipelineError> {

		let _atom = self.pipeline_lock.borrow_mut();

		let gl = self.context.as_ref();
		let (program, vertex_shader, fragment_shader) = unsafe {
			let program = gl.create_program()
				.map_err(|what|
					RenderPipelineError::ProgramCreationFailed { what })?;

			let vertex_shader = descriptor.vertex.shader.clone();
			gl.attach_shader(program, vertex_shader.as_raw_handle());

			let fragment_shader = match descriptor.fragment {
				Some(fragment_shader) => {
					let fragment_shader = fragment_shader.clone();
					gl.attach_shader(program, fragment_shader.as_raw_handle());

					Some(fragment_shader)
				},
				None => None
			};

			gl.link_program(program);
			if !gl.get_program_link_status(program) {
				let what = gl.get_program_info_log(program);
				return Err(RenderPipelineError::ProgramLinkFailed { what })
			} else if log_enabled!(log::Level::Debug) {
				let what = gl.get_program_info_log(program);
				if !what.is_empty() {
					debug!("Program linkage log: {}", what);
				}
			}

			(program, vertex_shader, fragment_shader)
		};

		Ok(RenderPipeline {
			inner: Rc::new(InnerRenderPipeline {
				context: self.context.clone(),
				access: Default::default(),
				program: unsafe { RenderProgram::new(gl, program) },
				vao: Default::default(),
				vertex_layout: From::from(descriptor.vertex.buffer),
				vertex_shader: VertexShader { inner: vertex_shader.inner.clone() },
				fragment_shader: fragment_shader.map(|fragment_shader|
					FragmentShader {
						inner: fragment_shader.inner.clone()
					}),
				primitive_state: descriptor.primitive_state,
				depth_stencil: descriptor.depth_stencil
			})
		})
	}

	instance_shader_creation_functions! {
		#[doc = "Tries to create a new vertex shader from the given source."]
		pub fn create_vertex_shader: VertexShader;
		#[doc = "Tries to create a new vertex shader from the given source."]
		pub fn create_fragment_shader: FragmentShader;
	}

	instance_initialized_buffer_creation_functions! {
		#[doc = "Tries to create a new vertex buffer with the given data."]
		pub fn create_vertex_buffer_with_data: VertexBuffer;
		#[doc = "Tries to create a new index buffer with the given data."]
		pub fn create_index_buffer_with_data: IndexBuffer;
		#[doc = "Tries to create a new uniform buffer with the given data."]
		pub fn create_uniform_buffer_with_data: UniformBuffer;
	}

	instance_zero_initialized_buffer_creation_functions! {
		#[doc = "Tries to create a new zero-initialized vertex buffer."]
		#[doc = "# Performance"]
		#[doc = "Creating zero-initialized buffers may involve an extra, "]
		#[doc = "zero-initialized allocation in host memory, as big as the "]
		#[doc = "target buffer on the device. Users should only sparringly "]
		#[doc = "rely on this function."]
		pub fn create_vertex_buffer: create_vertex_buffer_with_data -> VertexBuffer;
		#[doc = "Tries to create a new zero-initialized vertex buffer."]
		#[doc = "# Performance"]
		#[doc = "Creating zero-initialized buffers may involve an extra, "]
		#[doc = "zero-initialized allocation in host memory, as big as the "]
		#[doc = "target buffer on the device. Users should only sparringly "]
		#[doc = "rely on this function."]
		pub fn create_index_buffer: create_index_buffer_with_data -> IndexBuffer;
		#[doc = "Tries to create a new zero-initialized vertex buffer."]
		#[doc = "# Performance"]
		#[doc = "Creating zero-initialized buffers may involve an extra, "]
		#[doc = "zero-initialized allocation in host memory, as big as the "]
		#[doc = "target buffer on the device. Users should only sparringly "]
		#[doc = "rely on this function."]
		pub fn create_uniform_buffer: create_uniform_buffer_with_data -> UniformBuffer;
	}
}
