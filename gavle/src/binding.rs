use std::borrow::Cow;
use std::rc::Rc;
use crate::texture::{TextureFilter, Texture};
use crate::buffer::UniformBuffer;
use crate::access::AccessLock;
use glow::{Context, HasContext};
use std::convert::TryFrom;
use crate::RenderProgram;

/**  */
pub struct UniformGroup {
	/** Uniform binds. */
	pub(crate) entries: Rc<Vec<(String, OwnedUniformBind)>>
}
impl AccessLock for UniformGroup {
	fn acquire_write(&self) {
		panic!("tried to perform a write lock operation on a uniform buffer. \
			uniforms are read-only objects");
	}
	fn release_write(&self) {
		panic!("tried to perform a write lock operation on a uniform buffer. \
			uniforms are read-only objects");
	}
	fn acquire_read(&self) {
		for (_, entry) in &*self.entries {
			match entry {
				OwnedUniformBind::Texture { texture, .. } =>
					texture.acquire_read(),
				OwnedUniformBind::Buffer { buffer } =>
					buffer.acquire_read()
			}
		}
	}
	fn release_read(&self) {
		for (_, entry) in &*self.entries {
			match entry {
				OwnedUniformBind::Texture { texture, .. } =>
					texture.release_read(),
				OwnedUniformBind::Buffer { buffer } =>
					buffer.release_read()
			}
		}
	}
}
impl UniformGroup {
	/** Bind all of the elements of this uniform bind group.
	 *
	 * The correct shader program for this group must have already been bound
	 * into the pipeline by this point. */
	pub(crate) unsafe fn bind(
		&self,
		gl: &Context,
		program: &RenderProgram) {

		let mut allocator = Default::default();
		for (location, binder) in &*self.entries {
			binder.bind(
				gl,
				location.as_str(),
				program,
				&mut allocator)
		}
	}
}

/** Structure that manages allocations in the uniform binding groups. */
struct Allocator {
	/** Simple texture bumper. */
	texture: u32,
	/** Simple uniform bumper. */
	ubo: u32,
}
impl Allocator {
	/** Creates a new, empty allocator. */
	pub fn new() -> Self {
		Self {
			texture: 0,
			ubo: 0
		}
	}

	/** Acquire and mark the location of the next available texture slot, as an
	 * OpenGL enum value. */
	pub fn next_texture(&mut self) -> u32 {
		self.texture = self.texture.checked_add(1)
			.expect("tried to allocate more textures than there are 32 \
				bit unsigned integer values");
		self.texture - 1
	}

	/** Acquire and mark the location of the next available UBO binding slot, as
	 * an OpenGL-ready value. */
	pub fn next_ubo_binding(&mut self) -> u32 {
		self.ubo = self.ubo.checked_add(1)
			.expect("tried to allocate more textures than there are 32 \
				bit unsigned integer values");
		self.ubo - 1
	}
}
impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}

/** Owned internal version of the uniform bind specification structure. */
pub(crate) enum OwnedUniformBind {
	Buffer {
		/** Buffer object to be bound to this group. */
		buffer: UniformBuffer,
	},
	Texture {
		/** Texture object to be bound to this group. */
		texture: Texture,
		/** How this texture will be filtered when it needs to be downscaled. */
		far: TextureFilter,
		/** How this texture will be filtered when it needs to be upscaled. */
		near: TextureFilter,
	}
}
impl OwnedUniformBind {
	unsafe fn bind(
		&self,
		gl: &Context,
		target: &str,
		program: &RenderProgram,
		allocator: &mut Allocator) {

		match self {
			OwnedUniformBind::Buffer { buffer } => {
				let index = match gl.get_uniform_block_index(program.program, target) {
					Some(location) => location,
					None => {
						trace!("tried to bind to inactive uniform block at \
							\"{}\". data for this uniform will be missing",
							target);
						return
					}
				};

				let binding = allocator.next_ubo_binding();
				gl.uniform_block_binding(
					program.program,
					index,
					binding);

				gl.bind_buffer_range(
					glow::UNIFORM_BUFFER,
					binding,
					Some(buffer.inner.buffer),
					0,
					i32::try_from(buffer.len()).expect("buffer is \
						too big for shader use"));
			},
			OwnedUniformBind::Texture { texture, far, near } => {
				/* Check whether this target is active in the program. */
				if let None = program.uniforms.get(target) {
					trace!("tried to bind to the inactive uniform \"{}\". data \
						for this uniform will be missing", target);
					return
				}

				let location = match gl.get_uniform_location(program.program, target) {
					Some(location) => location,
					None => panic!("expected a uniform at \"{}\", found none",
						target)
				};

				let slot = allocator.next_texture();
				gl.active_texture(glow::TEXTURE0 + slot);
				gl.bind_texture(glow::TEXTURE_2D, Some(texture.inner.texture));
				gl.tex_parameter_i32(
					glow::TEXTURE_2D,
					glow::TEXTURE_MAG_FILTER,
					i32::try_from(near.as_opengl()).unwrap());
				gl.tex_parameter_i32(
					glow::TEXTURE_2D,
					glow::TEXTURE_MIN_FILTER,
					i32::try_from(far.as_opengl()).unwrap());

				gl.uniform_1_i32(
					Some(&location),
					i32::try_from(slot).unwrap());
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct UniformGroupDescriptor<'a> {
	/** List of entries for the uniform group. */
	pub entries: &'a [UniformGroupEntry<'a>]
}

#[derive(Debug, Clone)]
pub struct UniformGroupEntry<'a> {
	/** Name of the binding of this uniform in the shader program. */
	pub binding: Cow<'a, str>,
	/** Type of shader binding this entry refers to. */
	pub kind: UniformBind<'a>
}

#[derive(Debug, Copy, Clone)]
pub enum UniformBind<'a> {
	Buffer {
		/** Buffer object to be bound to this group. */
		buffer: &'a UniformBuffer,
	},
	Texture {
		/** Texture object to be bound to this group. */
		texture: &'a Texture,
		/** How this texture will be filtered when it needs to be downscaled. */
		far: TextureFilter,
		/** How this texture will be filtered when it needs to be upscaled. */
		near: TextureFilter,
	}
}
