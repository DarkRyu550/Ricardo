/** Four by four matrix type.
 *
 * This type exposes multiplication and transformation functionality for use in
 * game code. These matrices support rectilinear projection operations as well
 * as three-dimensional affine transformation operations.
 *
 * The layout of this structure is compatible with both the `std140` and
 * `std430` GLSL layouts, together with being marked as a POD structure, which
 * allows it to be copied directly into a device buffer. */
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
pub struct Matrix4([f32; 16]);

impl Matrix4 {
	/** Creates a new matrix with the given row-major layout array. */
	pub fn from_row_major_array(array: [f32; 16]) -> Self {
		Self(array)
	}

	/** Get the contents of this matrix as a row-major layour array. */
	pub fn as_row_major_array(&self) -> &[f32; 16] {
		&self.0
	}

	/** Creates a new identity matrix. This matrix corresponds to an identity
	 * affine transformation which leaves all points unchanged. */
	pub fn identity() -> Self {
		Self([
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		])
	}

	/** Creates a new axis-aligned three-dimensional scaling transformation with
	 * the given parameters for each of the axes. */
	pub fn scale(x: f32, y: f32, z: f32) -> Self {
		Self([
			  x, 0.0, 0.0, 0.0,
			0.0,   y, 0.0, 0.0,
			0.0, 0.0,   z, 0.0,
			0.0, 0.0, 0.0, 1.0
		])
	}

	/** Creates a new axis-aligned three-dimensional translation transformation
	 * with the given offsets for each of the axes. */
	pub fn translate(x: f32, y: f32, z: f32) -> Self {
		Self([
			1.0, 0.0, 0.0,   x,
			0.0, 1.0, 0.0,   y,
			0.0, 0.0, 1.0,   z,
			0.0, 0.0, 0.0, 1.0
		])
	}

	/** Creates a new transformation with which  */
	pub fn rectilinear_projection(fovy: f32, aspect: f32, n: f32, f: f32) -> Self {
		let z = -f / (n - f);
		let c = f * n / (n - f);

		let f = 1.0 / f32::tan(fovy / 2.0);
		let x = f / aspect;
		Self([
			  x, 0.0,  0.0, 0.0,
			0.0,   f,  0.0, 0.0,
			0.0, 0.0,    z,   c,
			0.0, 0.0,  1.0, 0.0,
		])
	}

	/** Creates a new axis-angle rotation transformation with the given pivot
	 * vector and rotation angle, given in radians. */
	pub fn rotate(x: f32, y: f32, z: f32, angle: f32) -> Self {
		/* Normalize the vector if needed. */
		let (x, y, z) = {
			let len = f32::sqrt(x * x + y * y + z * z);
			(x / len, y / len, z / len)
		};

		let sin = f32::sin(angle);
		let cos = f32::cos(angle);
		let ics = 1.0 - cos;

		let a = Self([
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
		]) * cos;
		let b = Self([
			x * x, x * y, x * z, 0.0,
			y * x, y * y, y * z, 0.0,
			z * x, z * y, z * z, 0.0,
			  0.0,   0.0,   0.0, 0.0
		]) * ics;
		let c = Self([
			0.0,  -z,   y, 0.0,
			  z, 0.0,  -x, 0.0,
			 -y,   x, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
		]) * sin;
		let d = Self([
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]);

		(a + b + c + d).transpose()
	}

	/** Transpose this matrix. */
	pub fn transpose(mut self) -> Self {
		let a = |i: usize, j: usize| i * 4 + j;

		for i in 0..4 {
			for j in 0..i {
				let x = self.0[a(i, j)];
				let y = self.0[a(j, i)];

				self.0[a(i, j)] = y;
				self.0[a(j, i)] = x;
			}
		}

		self
	}

	/** Find the value of the determinant of this matrix. */
	pub fn det(&self) -> f32 {
		let [
			a11, a12, a13, a14,
			a21, a22, a23, a24,
			a31, a32, a33, a34,
			a41, a42, a43, a44,
		] = self.0;

		let x0 = (a22 * a33 * a44) + (a23 * a34 * a42) + (a24 * a32 * a43);
		let x1 = (a24 * a33 * a42) + (a23 * a32 * a44) + (a22 * a34 * a43);
		let y0 = (a12 * a33 * a44) + (a13 * a34 * a42) + (a14 * a32 * a43);
		let y1 = (a14 * a33 * a42) + (a13 * a32 * a44) + (a12 * a34 * a43);
		let z0 = (a12 * a23 * a44) + (a13 * a24 * a42) + (a14 * a22 * a43);
		let z1 = (a14 * a23 * a42) + (a13 * a22 * a44) + (a12 * a24 * a43);
		let w0 = (a12 * a23 * a34) + (a13 * a24 * a32) + (a14 * a22 * a33);
		let w1 = (a14 * a23 * a32) + (a13 * a22 * a34) + (a12 * a24 * a33);

		let x = x0 - x1;
		let y = y0 - y1;
		let z = z0 - z1;
		let w = w0 - w1;

		(a11 * x) - (a21 * y) + (a31 * z) - (a41 * w)
	}
}
impl Default for Matrix4 {
	fn default() -> Self {
		Self::identity()
	}
}

/** Implementation of the standard matrix sum functionality. */
impl std::ops::Add for Matrix4 {
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		let iter = self.0.iter_mut().zip(&rhs.0);
		for (i, j) in iter { *i += *j; }
		self
	}
}

/** Assigning addition of one matrix by another. */
impl std::ops::AddAssign for Matrix4 {
	fn add_assign(&mut self, rhs: Self) {
		let iter = self.0.iter_mut().zip(&rhs.0);
		for (i, j) in iter { *i += *j; }
	}
}

/** Implementation of the standard matrix subtraction functionality. */
impl std::ops::Sub for Matrix4 {
	type Output = Self;

	fn sub(mut self, rhs: Self) -> Self::Output {
		let iter = self.0.iter_mut().zip(&rhs.0);
		for (i, j) in iter { *i -= *j; }
		self
	}
}

/** Assigning subtraction of one matrix by another. */
impl std::ops::SubAssign for Matrix4 {
	fn sub_assign(&mut self, rhs: Self) {
		let iter = self.0.iter_mut().zip(&rhs.0);
		for (i, j) in iter { *i -= *j; }
	}
}

/** Implementation of the multiplication of a matrix by a scalar value. */
impl std::ops::Mul<f32> for Matrix4 {
	type Output = Self;

	fn mul(mut self, rhs: f32) -> Self::Output {
		for i in &mut self.0 { *i *= rhs; }
		self
	}
}

/** Assigning multiplication of a matrix by a scalar. */
impl std::ops::MulAssign<f32> for Matrix4 {
	fn mul_assign(&mut self, rhs: f32) {
		for i in &mut self.0 { *i *= rhs; }
	}
}

/** Implementation of the division of a matrix by a scalar value. */
impl std::ops::Div<f32> for Matrix4 {
	type Output = Self;

	fn div(mut self, rhs: f32) -> Self::Output {
		for i in &mut self.0 { *i /= rhs; }
		self
	}
}

/** Assigning division of a matrix by a scalar. */
impl std::ops::DivAssign<f32> for Matrix4 {
	fn div_assign(&mut self, rhs: f32) {
		for i in &mut self.0 { *i /= rhs; }
	}
}

/** Implementation of standard matrix multiplication functionality. */
impl std::ops::Mul for Matrix4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let a = |i: usize, j: usize| self.0[i * 4 + j];
		let b = |i: usize, j: usize| rhs.0[i * 4 + j];

		Self([
			(a(0, 0) * b(0, 0)) + (a(0, 1) * b(1, 0)) + (a(0, 2) * b(2, 0)) + (a(0, 3) * b(3, 0)),
			(a(0, 0) * b(0, 1)) + (a(0, 1) * b(1, 1)) + (a(0, 2) * b(2, 1)) + (a(0, 3) * b(3, 1)),
			(a(0, 0) * b(0, 2)) + (a(0, 1) * b(1, 2)) + (a(0, 2) * b(2, 2)) + (a(0, 3) * b(3, 2)),
			(a(0, 0) * b(0, 3)) + (a(0, 1) * b(1, 3)) + (a(0, 2) * b(2, 3)) + (a(0, 3) * b(3, 3)),
			(a(1, 0) * b(0, 0)) + (a(1, 1) * b(1, 0)) + (a(1, 2) * b(2, 0)) + (a(1, 3) * b(3, 0)),
			(a(1, 0) * b(0, 1)) + (a(1, 1) * b(1, 1)) + (a(1, 2) * b(2, 1)) + (a(1, 3) * b(3, 1)),
			(a(1, 0) * b(0, 2)) + (a(1, 1) * b(1, 2)) + (a(1, 2) * b(2, 2)) + (a(1, 3) * b(3, 2)),
			(a(1, 0) * b(0, 3)) + (a(1, 1) * b(1, 3)) + (a(1, 2) * b(2, 3)) + (a(1, 3) * b(3, 3)),
			(a(2, 0) * b(0, 0)) + (a(2, 1) * b(1, 0)) + (a(2, 2) * b(2, 0)) + (a(2, 3) * b(3, 0)),
			(a(2, 0) * b(0, 1)) + (a(2, 1) * b(1, 1)) + (a(2, 2) * b(2, 1)) + (a(2, 3) * b(3, 1)),
			(a(2, 0) * b(0, 2)) + (a(2, 1) * b(1, 2)) + (a(2, 2) * b(2, 2)) + (a(2, 3) * b(3, 2)),
			(a(2, 0) * b(0, 3)) + (a(2, 1) * b(1, 3)) + (a(2, 2) * b(2, 3)) + (a(2, 3) * b(3, 3)),
			(a(3, 0) * b(3, 0)) + (a(3, 1) * b(1, 0)) + (a(3, 2) * b(2, 0)) + (a(3, 3) * b(3, 0)),
			(a(3, 0) * b(3, 1)) + (a(3, 1) * b(1, 1)) + (a(3, 2) * b(2, 1)) + (a(3, 3) * b(3, 1)),
			(a(3, 0) * b(3, 2)) + (a(3, 1) * b(1, 2)) + (a(3, 2) * b(2, 2)) + (a(3, 3) * b(3, 2)),
			(a(3, 0) * b(3, 3)) + (a(3, 1) * b(1, 3)) + (a(3, 2) * b(2, 3)) + (a(3, 3) * b(3, 3)),
		])
	}
}

/** Implementation of standard matrix multiplication functionality. */
impl std::ops::MulAssign for Matrix4 {
	fn mul_assign(&mut self, rhs: Self) {
		let a = |i: usize, j: usize| self.0[i * 4 + j];
		let b = |i: usize, j: usize| rhs.0[i * 4 + j];

		*self = Self([
			(a(0, 0) * b(0, 0)) + (a(0, 1) * b(1, 0)) + (a(0, 2) * b(2, 0)) + (a(0, 3) * b(3, 0)),
			(a(0, 0) * b(0, 1)) + (a(0, 1) * b(1, 1)) + (a(0, 2) * b(2, 1)) + (a(0, 3) * b(3, 1)),
			(a(0, 0) * b(0, 2)) + (a(0, 1) * b(1, 2)) + (a(0, 2) * b(2, 2)) + (a(0, 3) * b(3, 2)),
			(a(0, 0) * b(0, 3)) + (a(0, 1) * b(1, 3)) + (a(0, 2) * b(2, 3)) + (a(0, 3) * b(3, 3)),
			(a(1, 0) * b(0, 0)) + (a(1, 1) * b(1, 0)) + (a(1, 2) * b(2, 0)) + (a(1, 3) * b(3, 0)),
			(a(1, 0) * b(0, 1)) + (a(1, 1) * b(1, 1)) + (a(1, 2) * b(2, 1)) + (a(1, 3) * b(3, 1)),
			(a(1, 0) * b(0, 2)) + (a(1, 1) * b(1, 2)) + (a(1, 2) * b(2, 2)) + (a(1, 3) * b(3, 2)),
			(a(1, 0) * b(0, 3)) + (a(1, 1) * b(1, 3)) + (a(1, 2) * b(2, 3)) + (a(1, 3) * b(3, 3)),
			(a(2, 0) * b(0, 0)) + (a(2, 1) * b(1, 0)) + (a(2, 2) * b(2, 0)) + (a(2, 3) * b(3, 0)),
			(a(2, 0) * b(0, 1)) + (a(2, 1) * b(1, 1)) + (a(2, 2) * b(2, 1)) + (a(2, 3) * b(3, 1)),
			(a(2, 0) * b(0, 2)) + (a(2, 1) * b(1, 2)) + (a(2, 2) * b(2, 2)) + (a(2, 3) * b(3, 2)),
			(a(2, 0) * b(0, 3)) + (a(2, 1) * b(1, 3)) + (a(2, 2) * b(2, 3)) + (a(2, 3) * b(3, 3)),
			(a(3, 0) * b(3, 0)) + (a(3, 1) * b(1, 0)) + (a(3, 2) * b(2, 0)) + (a(3, 3) * b(3, 0)),
			(a(3, 0) * b(3, 1)) + (a(3, 1) * b(1, 1)) + (a(3, 2) * b(2, 1)) + (a(3, 3) * b(3, 1)),
			(a(3, 0) * b(3, 2)) + (a(3, 1) * b(1, 2)) + (a(3, 2) * b(2, 2)) + (a(3, 3) * b(3, 2)),
			(a(3, 0) * b(3, 3)) + (a(3, 1) * b(1, 3)) + (a(3, 2) * b(2, 3)) + (a(3, 3) * b(3, 3)),
		])
	}
}