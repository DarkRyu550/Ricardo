
pub mod mountains {
	use gavle::ShaderSource;
	use std::borrow::Cow;

	pub const VERTEX: ShaderSource<'static> =
		ShaderSource::Glsl(Cow::Borrowed(include_str!("shapes/Mountains.glsl")));
	pub const FRAGMENT: ShaderSource<'static> =
		ShaderSource::Glsl(Cow::Borrowed(include_str!("lighting/VertexColoredDirect.glsl")));
}

pub mod snowfall {
	use gavle::ShaderSource;
	use std::borrow::Cow;

	pub const VERTEX: ShaderSource<'static> =
		ShaderSource::Glsl(Cow::Borrowed(include_str!("shapes/Snowfall.glsl")));
	pub const FRAGMENT: ShaderSource<'static> =
		ShaderSource::Glsl(Cow::Borrowed(include_str!("lighting/VertexColoredDirect.glsl")));
}
