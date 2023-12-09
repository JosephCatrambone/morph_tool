
// This blog post was referenced for guidance https://khanhha.github.io/posts/Thin-Plate-Splines-Warping/
// But ultimately the source located here was ported: https://raw.githubusercontent.com/raphaelreme/tps/main/tps/thin_plate_spline.py

use ndarray::prelude::*;
use ndarray::*;
use ndarray_linalg::{Solve, SVD};

pub struct ThinPlateSpline {
	parameters: Array2<f32>,
	control_points: Array2<f32>,
}

impl ThinPlateSpline {

	/// Given two arrays of points in the form [x, y, x, y, x, y, ...] and an alpha,
	/// compute the thin-plate-spline solution and return an instance of the structure.
	/// Note that the order of x,y in the vec doesn't really matter as long as it's consistent and
	/// each point is contiguous.
	pub fn new(source_points: &Vec<f32>, destination_points: &Vec<f32>, alpha: f32) -> Self {
		assert_eq!(source_points.len() % 2, 0);
		assert_eq!(destination_points.len() % 2, 0);
		assert!(source_points.len() > 3);
		assert!(destination_points.len() > 3);
		assert_eq!(source_points.len(), destination_points.len());

		let source_mat = vec_to_mat(source_points, source_points.len()/2, 2);
		let dest_mat = vec_to_mat(destination_points, destination_points.len()/2, 2);

		let n_c = source_points.len() / 2;
		let d_s = 2; // We're always assuming (x, y) for each point, but this could generalize.

		let radial_distances = compute_radial_distances(&source_mat, &dest_mat);

		// Build K, X'_c, X'_c^T, and 0.
		// Build A from the above.
		// Build AP = Y and solve.

		// Top left, K
		let K = radial_distances + (Array2::<f32>::eye(n_c) * alpha);
		// Top right, [ 1 | source ] or X_p
		let X_p = concatenate(Axis(1), &[(&Array2::ones((n_c, 1))).into(), (&source_mat).into()]).expect("Source dimension mismatch for source points.");
		// Bottom left: X_p.T
		let X_p_t = X_p.t(); // .reversed_axes for by-move.
		// Bottom right:
		let zeros = Array2::zeros((d_s + 1, d_s + 1));
		let A_top = concatenate(Axis(1), &[(&K).into(), (&X_p).into()]).unwrap(); // hstack
		let A_bottom = concatenate(Axis(1), &[(&X_p_t).into(), (&zeros).into()]).unwrap();
		let A = concatenate(Axis(0), &[(&A_top).into(), (&A_bottom).into()]).unwrap(); // vstack

		// Build Y from the destinations and some zero fills.
		let target_zeros = Array2::zeros((d_s + 1, dest_mat.ncols()));
		let y = concatenate(Axis(0), &[(&dest_mat).into(), (&target_zeros).into()]).unwrap();

		let parameters = least_squares(&A, &y).unwrap();

		Self {
			parameters,
			control_points: source_mat.into(),
		}
	}

	pub fn transform(&self, points: &Vec<f32>) -> Vec<f32> {
		let pts = vec_to_mat(points, points.len()/2, 2);
		let phi = compute_radial_distances(&self.control_points, &pts);
		let augmented = concatenate(Axis(1), &[(&phi).into(), (&Array2::ones((pts.shape()[0], 1))).into(), (&pts).into()]).expect("Shape mismatch in concatenated matrix.");
		let result = augmented.dot(&self.parameters);
		result.into_raw_vec()
	}
}

fn vec_to_mat(points: &Vec<f32>, num_rows: usize, num_columns: usize) -> Array2<f32> {
	Array2::from_shape_fn((num_rows, num_columns), |(i, j)| {
		points[num_columns*i + j]
	})
}

fn least_squares(a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>, ndarray_linalg::error::LinalgError> {
	// [Gilbert Strang appears as a force-ghost projection]
	// Remember your linear algebra.
	// Ax = b
	// USV x = b
	// To invert noninvertible S, let the diagonals be zeros if they're close to zero or 1/diag otherwise.
	// SV x = U^t b
	// V x = S^-1 U^t b
	// x = V S^-1 U^t b
	/*
	In [42]: x
	Out[42]:
	array([[-1. ,  2.2],
	       [ 4.2,  6.9]])

	In [43]: v @ numpy.linalg.inv(numpy.diag(s)) @ u.T[:2,:] @ b
	Out[43]:
	array([[-0.99826941,  2.20432477],
	       [ 4.20851867,  6.90574251]])
	*/
	let (u, mut s, v) = a.svd(true, true)?;
	let u = u.expect("calc_u is true but u is not present!?");
	let v = v.expect("calc_v is true but v is not present!?");
	// s should have only two elements, but...
	s.iter_mut().for_each(|value: &mut f32| { *value = if *value < 1e-6f32 { 0.0 } else { 1.0f32 / *value }; } );

	// Ax = b -- (Ax) is in R^(p by q) so b is in R^(p by q)
	// So A and b must have the same number of rows.  x and b must have the same number of columns.
	// Since we also know A and x multiply, A has the same number of columns as x has rows.
	// Reminder: x = V Sinv Ut b
	// V_rows must have x_rows.  b_cols must have x_cols.
	//Ok(v.dot(&Array2::from_diag(&s)).dot(&(u.t().slice(s![..2, ..]))).dot(b))
	let v_sinv = v.t().slice(s![..a.ncols(), ..]).dot(&Array2::from_diag(&s));
	// b is a given, so Ut cols must equal b rows.
	let v_sinv_ut = v_sinv.dot(&(u.t().slice(s![..v_sinv.ncols(), ..b.nrows()])));
	Ok(v_sinv_ut.dot(b))
}

fn compute_radial_distances(control: &Array2<f32>, pts: &Array2<f32>) -> Array2<f32> {
	let distances = compute_pairwise_distances(pts, control);
	distances.mapv(|value| {
		if value > 1e-5 {
			(value*value)*value.log10()
		} else {
			0.0f32
		}
	})
}

fn compute_pairwise_distances(a: &Array2<f32>, b: &Array2<f32>) -> Array2<f32> {
	assert_eq!(a.ncols(), b.ncols());
	Array::from_shape_fn((a.nrows(), b.nrows()), |(i, j)| {
		let mut delta = 0.0f32;
		for k in 0..a.ncols() {
			let d = a[(i, k)] - b[(j, k)];
			delta += d*d;
		}
		delta.sqrt()
	})
}


#[cfg(test)]
mod tests {
	use ndarray_linalg::{assert_aclose, assert_close_l1};
	use super::*;
	use rand::prelude::*;

	#[test]
	fn test_distance() {
		let a = Array2::from_shape_vec((3, 2), vec![
			0.0f32, 0.0,
			3.0, 4.0,
			10.0, 10.0
		]).unwrap();
		let origin = Array2::zeros((1, 2));

		let distances = compute_pairwise_distances(&origin, &a);
		assert_eq!(distances[(0, 0)], 0.0);
		assert_eq!(distances[(0, 1)], 5.0);
		assert!((distances[(0, 2)] - 200.0_f32.sqrt()).abs() < 1e-5);
	}

	#[test]
	fn test_to_mat() {
		let values = vec![0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0];
		let out = vec_to_mat(&values, 3, 2);
		assert_aclose!(out[[0, 0]], 0.0, 1e-5);
		assert_aclose!(out[[0, 1]], 1.0, 1e-5);
		assert_aclose!(out[[1, 0]], 2.0, 1e-5);
		assert_aclose!(out[[1, 1]], 3.0, 1e-5);
		assert_aclose!(out[[2, 0]], 4.0, 1e-5);
		assert_aclose!(out[[2, 1]], 5.0, 1e-5);
	}

	#[test]
	fn test_lstsq() {
		let mut rng = thread_rng();
		let a = Array2::from_shape_fn((10, 4), |(i, j)| {
			rng.gen::<f32>()
		});
		let x = Array2::from_shape_fn((4, rng.gen_range(2usize..5)), |(i, j)| {
			rng.gen::<f32>()
		});

		let b = a.dot(&x);

		let x_recovered = least_squares(&a, &b).unwrap();
		assert!(x.abs_diff_eq(&x_recovered, 1e-5));
	}

	#[test]
	fn test_transform_simple() {
		let src_points = vec![
			0.0f32, 0.0,
			8.0, 0.0,
			0.0, 10.0,
		];
		let dst_points = vec![
			10.0f32, 10.0,
			18.0, 10.0,
			10.0, 18.0,

		];
		let tps = ThinPlateSpline::new(&src_points, &dst_points, 0.1f32);

		let transformed = tps.transform(&vec![
			0.0, 0.0,
			4.0, 0.0,
			8.0, 0.0,
			0.0, 5.0,
			0.0, 10.0,
			4.0, 5.0,
		]);
		let expected = vec![
			10.0, 10.0, // Exact src to dest so should get the universal offset of (10, 10)
			14.0, 10.0, // Half way between src and dest.  4 is half way between 0 and 8 + 10 offset.
			18.0, 10.0, // Exact src/dst match.
			10.0, 14.0, // Half way on y axis between the 0 src and 10 dst.
			10.0, 18.0, // Exact src/dst match.
			14.0, 14.0, // Half way on each axis plus offset.
		];
		assert_close_l1!(&vec_to_mat(&transformed, 6, 2), &vec_to_mat(&expected, 6, 2), 1e-3);
	}
}