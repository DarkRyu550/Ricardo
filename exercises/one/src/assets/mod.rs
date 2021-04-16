
/** Shaders used in the example render pass. */
pub mod example {
	use gavle::ShaderSource;

	/** Vertex program of this shader. */
	pub fn vertex() -> ShaderSource<'static> {
		ShaderSource::Glsl(include_str!("example/vert.glsl").into())
	}

	/** Fragment program of this shader. */
	pub fn fragment() -> ShaderSource<'static> {
		ShaderSource::Glsl(include_str!("example/frag.glsl").into())
	}
}
