
/** Shaders used in the main render pass of the visitor. */
pub mod visitor {
	use gavle::ShaderSource;

	/** Vertex program of this shader. */
	pub fn vertex() -> ShaderSource<'static> {
		ShaderSource::Glsl(include_str!("visitor/vert.glsl").into())
	}

	/** Fragment program of this shader. */
	pub fn fragment() -> ShaderSource<'static> {
		ShaderSource::Glsl(include_str!("visitor/frag.glsl").into())
	}
}

/** Structures and assets related to the dish model. */
pub mod dish {
	/** Get a reference to the dish model as a Wavefront OBJ. */
	pub fn obj() -> &'static obj::Obj<obj::TexturedVertex, u32> {
		const SOURCE: &'static [u8] = include_bytes!("dish/dish.obj");

		static mut CACHE: Option<obj::Obj<obj::TexturedVertex, u32>> = None;
		static LOCK: std::sync::Once = std::sync::Once::new();

		unsafe {
			LOCK.call_once(|| {
				let mut vec = Default::default();
				let mut decoder = std::io::Cursor::new(SOURCE);

				std::io::Read::read_to_end(&mut decoder, &mut vec)
					.expect("bundled dish xz data is invalid");

				let obj = obj::load_obj(std::io::BufReader::new(&vec[..]))
					.expect("bundled dish obj data is invalid");

				CACHE = Some(obj);
			});
			CACHE.as_ref().unwrap()
		}
	}

	/** Decode the albedo texture data for the dish into a raw image buffer.
	 *
	 * Every call to this function will perform the decoding process into a new raw
	 * image allocation, given the data for the raw texture is too large to be
	 * cached on  the web, where we'll be competing with Facebook and YouTube for
	 * resources. */
	pub fn albedo() -> image::RgbaImage {
		image::load_from_memory(include_bytes!("dish/albedo.png"))
			.unwrap()
			.into_rgba8()
	}

	/** Decode the normal texture data for the dish into a raw image buffer.
	 *
	 * Every call to this function will perform the decoding process into a new raw
	 * image allocation, given the data for the raw texture is too large to be
	 * cached on  the web, where we'll be competing with Facebook and YouTube for
	 * resources. */
	pub fn normal() -> image::RgbaImage {
		image::load_from_memory(include_bytes!("dish/normal.jpg"))
			.unwrap()
			.into_rgba8()
	}

	/** Decode the roughness texture data for the dish into a raw image buffer.
	 *
	 * Every call to this function will perform the decoding process into a new raw
	 * image allocation, given the data for the raw texture is too large to be
	 * cached on  the web, where we'll be competing with Facebook and YouTube for
	 * resources. */
	pub fn roughness() -> image::RgbaImage {
		image::load_from_memory(include_bytes!("dish/roughness.jpg"))
			.unwrap()
			.into_rgba8()
	}

	/** Decode the metallic texture data for the dish into a raw image buffer.
	 *
	 * Every call to this function will perform the decoding process into a new raw
	 * image allocation, given the data for the raw texture is too large to be
	 * cached on  the web, where we'll be competing with Facebook and YouTube for
	 * resources. */
	pub fn metallic() -> image::RgbaImage {
		image::load_from_memory(include_bytes!("dish/metallic.jpg"))
			.unwrap()
			.into_rgba8()
	}
}
