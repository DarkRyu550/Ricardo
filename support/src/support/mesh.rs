use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use smallvec::SmallVec;
use crate::support::Vertex;
use std::num::TryFromIntError;
use tinyvec::ArrayVec;

pub struct Mesh {
	vertices: Vec<Vertex>,
	indices: Vec<u32>
}
impl Mesh {
	/** Load the data for this mesh from the given object file. */
	pub fn from_obj(model: &obj::Obj<obj::TexturedVertex, u32>)
		-> Result<Self, InvalidMesh> {

		/** Vertex type that implements full order and equality. */
		#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
		struct Vertex {
			position: [OrderedFloat<f32>; 3],
			normal:   [OrderedFloat<f32>; 3],
			texture:  [OrderedFloat<f32>; 3],
		}
		impl From<obj::TexturedVertex> for Vertex {
			fn from(vert: obj::TexturedVertex) -> Self {
				Self {
					position: [
						vert.position[0].into(),
						vert.position[1].into(),
						vert.position[2].into()
					],
					normal: [
						vert.normal[0].into(),
						vert.normal[1].into(),
						vert.normal[2].into()
					],
					texture: [
						vert.texture[0].into(),
						vert.texture[1].into(),
						vert.texture[2].into()
					]
				}
			}
		}

		/** Three vertices of a triangulated face in the model. */
		struct Face<'a> {
			vert0: &'a obj::TexturedVertex,
			vert1: &'a obj::TexturedVertex,
			vert2: &'a obj::TexturedVertex,

			/** Surface-wide normal. */
			normal: [f32; 3],
			/** Tangent vector of the face. */
			tangent: [f32; 3],
			/** Bitangent vector of the face. */
			bitangent: [f32; 3]
		}

		let mut global_faces = Vec::with_capacity(model.indices.len() / 3);
		let mut vertices = BTreeMap::new();

		/* Build the list of elements. */
		for triplet in model.indices.chunks_exact(3) {
			let vert0 = usize::try_from(triplet[0])
				.map_err(|what| InvalidMesh::InnumerableVertices { what })?;
			let vert1 = usize::try_from(triplet[1])
				.map_err(|what| InvalidMesh::InnumerableVertices { what })?;
			let vert2 = usize::try_from(triplet[2])
				.map_err(|what| InvalidMesh::InnumerableVertices { what })?;

			let vert0 = &model.vertices[vert0];
			let vert1 = &model.vertices[vert1];
			let vert2 = &model.vertices[vert2];

			let normal = {
				let x = (vert0.normal[0] + vert1.normal[0] + vert2.normal[0]) / 3.0;
				let y = (vert0.normal[1] + vert1.normal[1] + vert2.normal[1]) / 3.0;
				let z = (vert0.normal[2] + vert1.normal[2] + vert2.normal[2]) / 3.0;

				let l = f32::sqrt(x.powf(2.0) + y.powf(2.0) + z.powf(2.0));
				if l == 0.0 {
					/* This surface normal is null, meaning that this is an
					 * invalid triangle. Give up on the mesh. */
					return Err(InvalidMesh::NullSurfaceNormal)
				}

				let x = x / l;
				let y = y / l;
				let z = z / l;

				[x, y, z]
			};
			let (tangent, bitangent) = {
				let edge0 = [
					vert1.position[0] - vert0.position[0],
					vert1.position[1] - vert0.position[1],
					vert1.position[2] - vert0.position[2]];
				let edge1 = [
					vert2.position[0] - vert0.position[0],
					vert2.position[1] - vert0.position[1],
					vert2.position[2] - vert0.position[2]];

				let uv0 = [vert1.texture[0] - vert0.texture[0], vert1.texture[1] - vert0.texture[1]];
				let uv1 = [vert2.texture[0] - vert0.texture[0], vert2.texture[1] - vert0.texture[1]];

				let edge_cross =
					  (edge0[1] * edge1[2] - edge0[2] * edge1[1]).powf(2.0)
					+ (edge0[2] * edge1[0] - edge0[0] * edge1[2]).powf(2.0)
					+ (edge0[0] * edge1[1] - edge0[1] * edge1[0]).powf(2.0);
				let uv_cross = uv0[0] * uv1[1] - uv0[1] * uv1[0];

				if edge_cross == 0.0 || uv_cross == 0.0 {
					/* This is a degenerate triangle, we can't really calculate the
					 * tangent direction for it, so we just give up. */
					return Err(InvalidMesh::DegenerateTriangle {
						vertex0: *vert0,
						vertex1: *vert1,
						vertex2: *vert2,
						edge_cross,
						uv_cross
					})
				} else {
					/* Calculate the tangent vectors. */
					let base = 1.0 / uv_cross;
					let tangent = [
						base * (uv1[1] * edge0[0] - uv0[1] * edge1[0]),
						base * (uv1[1] * edge0[1] - uv0[1] * edge1[1]),
						base * (uv1[1] * edge0[2] - uv0[1] * edge1[2])
					];
					let bitangent = [
						base * (-uv1[0] * edge0[0] + uv0[0] * edge1[0]),
						base * (-uv1[0] * edge0[1] + uv0[0] * edge1[1]),
						base * (-uv1[0] * edge0[2] + uv0[0] * edge1[2]),
					];

					(tangent, bitangent)
				}
			};

			/* Add the newly created face to the bank of faces. */
			global_faces.push(Face {
				vert0,
				vert1,
				vert2,
				normal,
				tangent,
				bitangent
			});

			/* Register the newly added face to the vertex lookup table. */
			vertices.entry(Vertex::from(*vert0))
				.or_insert_with(SmallVec::<[usize; 32]>::new)
				.push(global_faces.len() - 1);
			vertices.entry(Vertex::from(*vert1))
				.or_insert_with(SmallVec::<[usize; 32]>::new)
				.push(global_faces.len() - 1);
			vertices.entry(Vertex::from(*vert2))
				.or_insert_with(SmallVec::<[usize; 32]>::new)
				.push(global_faces.len() - 1);
		}

		/* Build a new, stably allocated and sorted array of vertices array that
		 * can be search through a binary search and that will later be used to
		 * generate the indices for the faces. */
		let vertex_array = vertices
			.keys()
			.cloned()
			.collect::<Vec<_>>();

		/* Generate the index buffer. */
		let indices = global_faces.iter()
			.map(|face| {
				let vert0 = Vertex::from(*face.vert0);
				let vert1 = Vertex::from(*face.vert1);
				let vert2 = Vertex::from(*face.vert2);

				let vert0 = u32::try_from(vertex_array.binary_search(&vert0).unwrap())
					.map_err(|what| InvalidMesh::InnumerableVertices { what })?;
				let vert1 = u32::try_from(vertex_array.binary_search(&vert1).unwrap())
					.map_err(|what| InvalidMesh::InnumerableVertices { what })?;
				let vert2 = u32::try_from(vertex_array.binary_search(&vert2).unwrap())
					.map_err(|what| InvalidMesh::InnumerableVertices { what })?;

				let array = ArrayVec::<[u32; 3]>::from([vert0, vert1, vert2]);
				Ok(array.into_iter())
			})
			.collect::<Result<Vec<_>, _>>()?
			.into_iter()
			.flat_map(|iter| iter)
			.collect::<Vec<_>>();

		/* Generate the vertices. */
		let vertices = vertices.into_iter()
			.map(|(vertex, faces)| {
				/* Copy the first three parameters. */
				let position = [
					vertex.position[0].into_inner(),
					vertex.position[1].into_inner(),
					vertex.position[2].into_inner(),
				];
				let texture = [
					vertex.texture[0].into_inner(),
					vertex.texture[1].into_inner(),
				];
				let normal = [
					vertex.normal[0].into_inner(),
					vertex.normal[1].into_inner(),
					vertex.normal[2].into_inner(),
				];

				/* Find the mean of the other parameters from their faces, then
				 * normalize the vector space. */
				let (tangent, bitangent) = faces.iter()
					.map(|index| &global_faces[*index])
					.map(|face| (face.tangent, face.bitangent))
					.reduce(|(a, c), (b, d)| (
						[
							a[0] + b[0],
							a[1] + b[1],
							a[2] + b[2]
						],
						[
							c[0] + d[0],
							c[1] + d[1],
							c[2] + d[2]
						]
					))
					.map(|(a, b)| (
						[
							a[0] / faces.len() as f32,
							a[1] / faces.len() as f32,
							a[2] / faces.len() as f32,
						],
						[
							b[0] / faces.len() as f32,
							b[1] / faces.len() as f32,
							b[2] / faces.len() as f32,
						],
					))
					.unwrap();

				let ntb = {
					/* Normalize the NTB matrix. */
					let nl = f32::sqrt(normal[0].powf(2.0) + normal[1].powf(2.0) + normal[2].powf(2.0));
					let tl = f32::sqrt(tangent[0].powf(2.0) + tangent[1].powf(2.0) + tangent[2].powf(2.0));
					let bl = f32::sqrt(bitangent[0].powf(2.0) + bitangent[1].powf(2.0) + bitangent[2].powf(2.0));

					assert_ne!(nl, 0.0, "NTB normal vector length must not be zero at this point");
					assert_ne!(tl, 0.0, "NTB tangent vector length must not be zero at this point");
					assert_ne!(bl, 0.0, "NTB bitangent vector length must not be zero at this point");

					(
						[
							normal[0] / nl,
							normal[1] / nl,
							normal[2] / nl,
						],
						[
							tangent[0] / tl,
							tangent[1] / tl,
							tangent[2] / tl,
						],
						[
							bitangent[0] / bl,
							bitangent[1] / bl,
							bitangent[2] / bl,
						]
					)
				};

				/* Build the vertex. */
				self::Vertex::new_unchecked(
					position,
					texture,
					ntb.0,
					ntb.1,
					ntb.2)
			})
			.collect::<Vec<_>>();


		Ok(Self {
			vertices,
			indices
		})
	}

	/** Get a reference to the vertices in this mesh. */
	pub fn vertices(&self) -> &[Vertex] {
		&self.vertices
	}

	/** Get a reference to the indices in this mesh. */
	pub fn indices(&self) -> &[u32] {
		&self.indices
	}
}

/** Error types for invalid meshes. */
#[derive(Debug, thiserror::Error)]
pub enum InvalidMesh {
	#[error("The mesh contains a degenerate triangle. The length of the cross \
		product vector of the edge vectors is {edge_cross} and the lenth of \
		the cross vector of the texture vectors is {uv_cross}")]
	DegenerateTriangle {
		vertex0: obj::TexturedVertex,
		vertex1: obj::TexturedVertex,
		vertex2: obj::TexturedVertex,

		edge_cross: f32,
		uv_cross: f32,
	},
	#[error("The mesh contains a non-unitary normal value: {0:?}")]
	NonUnitaryNormal([f32; 3]),
	#[error("One of the calculated surface normals is a null vector")]
	NullSurfaceNormal,
	#[error("The number of vertices in the mesh would be larger than a u32: {what}")]
	InnumerableVertices { what: TryFromIntError }
}
