use gavle::*;
use std::borrow::Cow;
use crate::support::Matrix4;

/** Structure containing the data for a vertex in three-dimensional space. */
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, bytemuck::Zeroable, bytemuck::Pod, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Vertex {
	/** Position data in three-dimensional model space. */
	position: [f32; 3],
	/** Texture coordinate data in two-dimensional sampler space. */
	texture: [f32; 2],
	/** Color value associated with this vertex. */
	color: [f32; 3],
	/** Normal vector data in normalized three dimensional space. */
	normal: [f32; 3],
	/** Vector tangent to the normal and aligned to the texture plane. */
	tangent: [f32; 3],
	/** Vector tangent to both the normal and the tangent. */
	bitangent: [f32; 3],
}
impl Vertex {
	/** Layout of buffers that use this structure as their vertex type. */
	pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
		array_stride: 68,
		attributes: &[
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 0,
				binding: Cow::Borrowed("tt_vert_position")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Two,
				offset: 12,
				binding: Cow::Borrowed("tt_vert_texture")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 20,
				binding: Cow::Borrowed("tt_vert_color")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 32,
				binding: Cow::Borrowed("tt_vert_normal")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 44,
				binding: Cow::Borrowed("tt_vert_tangent")
			},
			VertexAttribute {
				kind: VertexType::F32,
				components: VertexComponents::Three,
				offset: 56,
				binding: Cow::Borrowed("tt_vert_bitangent")
			},
		]
	};

	/** Create a new vertex, checking for the validity of the parameters.
	 *
	 * In order to be valid, a vertex must satisfy the following conditions:
	 * 1. The `normal`, `tangent` and `bitangent` vectors must together form a
	 *    right-handed or left-handed orthonormal base in three-dimensional
	 *    space.
	 *
	 * # Panic
	 * This function panics if any of the requirements for a valid matrix are
	 * not met. If you want to deal with potentially invalid vertices, you
	 * should use the [`try_new()`] function, instead.
	 */
	pub fn new(
		position: [f32; 3],
		texture: [f32; 2],
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3]) -> Self {

		Self::try_new(position, texture, normal, tangent, bitangent).unwrap()
	}

	/** Create a new vertex, checking for the validity of the parameters.
	 *
	 * In order to be valid, a vertex must satisfy the following conditions:
	 * 1. The `normal`, `tangent` and `bitangent` vectors must together form a
	 *    right-handed or left-handed orthonormal base in three-dimensional
	 *    space.
	 */
	pub fn try_new(
		position: [f32; 3],
		texture: [f32; 2],
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3]) -> Result<Self, InvalidVertex> {

		/* Check whether the NTB vectors form an orthonormal base. */
		let ntb_determinant = Matrix4::from_row_major_array([
			normal[0], tangent[0], bitangent[0], 0.0,
			normal[1], tangent[1], bitangent[1], 0.0,
			normal[2], tangent[2], bitangent[2], 0.0,
			      0.0,        0.0,          0.0, 1.0,
		]).det();

		/* Tolerate a bit of numerical error. */
		if f32::round(ntb_determinant.abs() * 100.0) != 100.0 {
			return Err(InvalidVertex::NonOrthonormalNBTVectors {
				normal,
				tangent,
				bitangent,
				determinant: ntb_determinant
			})
		}

		Ok(Self {
			position,
			texture,
			color: [0.0; 3],
			normal,
			tangent,
			bitangent
		})
	}

	/** Create a new vertex. */
	pub const fn new_unchecked(
		position: [f32; 3],
		texture: [f32; 2],
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3]) -> Self {

		Self {
			position,
			texture,
			color: [0.0; 3],
			normal,
			tangent,
			bitangent
		}
	}

	/** Create a new vertex, checking for the validity of the parameters.
	 *
	 * In order to be valid, a vertex must satisfy the following conditions:
	 * 1. The `normal`, `tangent` and `bitangent` vectors must together form a
	 *    right-handed or left-handed orthonormal base in three-dimensional
	 *    space.
	 */
	pub fn try_new_with_color(
		position: [f32; 3],
		texture: [f32; 2],
		color: [f32; 3],
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3]) -> Result<Self, InvalidVertex> {

		/* Check whether the NTB vectors form an orthonormal base. */
		let ntb_determinant = Matrix4::from_row_major_array([
			normal[0], tangent[0], bitangent[0], 0.0,
			normal[1], tangent[1], bitangent[1], 0.0,
			normal[2], tangent[2], bitangent[2], 0.0,
			0.0,        0.0,          0.0, 1.0,
		]).det();

		/* Tolerate a bit of numerical error. */
		if f32::round(ntb_determinant.abs() * 100.0) != 100.0 {
			return Err(InvalidVertex::NonOrthonormalNBTVectors {
				normal,
				tangent,
				bitangent,
				determinant: ntb_determinant
			})
		}

		Ok(Self {
			position,
			texture,
			color,
			normal,
			tangent,
			bitangent
		})
	}

	/** Create a new vertex. */
	pub const fn new_unchecked_with_color(
		position: [f32; 3],
		texture: [f32; 2],
		color: [f32; 3],
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3]) -> Self {

		Self {
			position,
			texture,
			color,
			normal,
			tangent,
			bitangent
		}
	}

	/** Position data in three-dimensional model space. */
	pub fn position(&self) -> [f32; 3] {
		self.position
	}

	/** Texture coordinate data in two-dimensional sampler space. */
	pub fn texture(&self) -> [f32; 2] {
		self.texture
	}

	/** Color value in RGB color space. */
	pub fn color(&self) -> [f32; 3] {
		self.color
	}

	/** Normal vector data in normalized three dimensional space. */
	pub fn normal(&self) -> [f32; 3] {
		self.normal
	}

	/** Vector tangent to the normal and aligned to the texture plane. */
	pub fn tangent(&self) -> [f32; 3] {
		self.tangent
	}

	/** Vector tangent to both the normal and the tangent. */
	pub fn bitangent(&self) -> [f32; 3] {
		self.bitangent
	}
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidVertex {
	#[error("The normal ({normal:?}), tangent ({tangent:?}) and bitangent \
		({bitangent:?}) vectors in this vertex do not form an orthonormal \
		base, as the absolute value of the determinant is {determinant} \
		instead of 1.0")]
	NonOrthonormalNBTVectors {
		normal: [f32; 3],
		tangent: [f32; 3],
		bitangent: [f32; 3],
		determinant: f32,
	}
}
