use crate::support::Matrix4;

/** This structure allows for obtaining the matrix transformation from camera
 * parameters such as position, rotation and projection type. */
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Camera {
	/** The projection type to be applied, together with its parameters. */
	pub projection: Projection,
	/** The current position of the camera, in world space. */
	pub position: [f32; 3],
	/** The current yaw rotation angle of the camera, in radians. */
	pub yaw: f32,
	/** The current pitch rotation angle of the camera, in radians. */
	pub pitch: f32,
}
impl Camera {
	/** Calculate the composite camera transformation.
	 *
	 * In more technical terms, the camera transformation is responsible for
	 * taking in world space vertices and outputting projection-screen space
	 * coordinates. */
	pub fn matrix(&self, aspect: f32) -> Matrix4 {
		let matrix = Matrix4::identity();
		let matrix = Matrix4::translate(
			-self.position[0],
			-self.position[1],
			-self.position[2]) * matrix;
		let matrix = Matrix4::rotate(
			0.0,
			1.0,
			0.0,
			self.yaw) * matrix;
		let matrix = Matrix4::rotate(
			1.0,
			0.0,
			0.0,
			self.pitch) * matrix;
		let matrix = self.projection.matrix(aspect) * matrix;

		matrix
	}
}

/** Projection type to be applied by the camera.
 *
 * The camera has the job of transforming three dimensional world space vertices
 * into screen-space, effectively two-dimensional vertices. This enum describes
 * each supported way for the camera to do this, as well as specify their
 * parameters. */
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "Type")]
pub enum Projection {
	/** The rectilinear perspective projection.
	 *
	 * This is the projection most people associate with 3D games, fairly simple
	 * in mathematical terms but extremely fast to calculate and apply, its
	 * projection strategy is to scale down and move to the center of the screen
	 * vertices which are far away from the camera. */
	#[serde(rename_all = "PascalCase")]
	Perspective {
		/** The field of view angle, in radians. */
		field_of_view: f32,
		/** The distance from the position of the camera to the near clipping plane. */
		near: f32,
		/** The distance from the position of the camera to the far clipping plane. */
		far: f32,
	},
	/** The orthographic projection.
	 *
	 * This is arguably the simplest form of projection from the 3D world to the
	 * 2D projection plane: Do nothing. The orthographic projection simply
	 * normalizes a world cube into the screen space cube, performing no
	 * distortion operations at all. */
	#[serde(rename_all = "PascalCase")]
	Orthographic {
		/** Left side of the projection cube.
		 *
		 * This is measured as a horizontal distance from the position of the
		 * camera, taking into account its current rotation, so, horizontal in
		 * the perspective of the camera itself. */
		left: f32,
		/** Right side of the projection cube.
		 *
		 * This is measured as a horizontal distance from the position of the
		 * camera, taking into account its current rotation, so, horizontal in
		 * the perspective of the camera itself. */
		right: f32,
		/** Top side of the projection cube.
		 *
		 * This is measured as a vertical distance from the position of the
		 * camera, taking into account its current rotation, so, vertical in
		 * the perspective of the camera itself. */
		top: f32,
		/** Bottom side of the projection cube.
		 *
		 * This is measured as a vertical distance from the position of the
		 * camera, taking into account its current rotation, so, vertical in
		 * the perspective of the camera itself. */
		bottom: f32,
		/** Near face of the projection cube.
		 *
		 * This is measured as a distance in depth from the position of the
		 * camera, taking into account its current rotation, so, depth in the
		 * perspective of the camera itself. */
		near: f32,
		/** Far face of the projection cube.
		 *
		 * This is measured as a distance in depth from the position of the
		 * camera, taking into account its current rotation, so, depth in the
		 * perspective of the camera itself. */
		far: f32,
	}
}
impl Projection {
	/** Calculate the projection transformation. */
	fn matrix(&self, aspect: f32) -> Matrix4 {
		match self {
			Self::Perspective { field_of_view, far, near } =>
				Matrix4::rectilinear_projection(
					*field_of_view,
					aspect,
					*near,
					*far),
			Self::Orthographic { left, right, top, bottom, near, far } =>
				Matrix4::orthographic_projection(
					*left, *right,
					*top, *bottom,
					*near, *far)
		}
	}
}
