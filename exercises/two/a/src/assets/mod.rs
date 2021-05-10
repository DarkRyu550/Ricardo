
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
