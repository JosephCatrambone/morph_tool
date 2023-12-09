
// This blog post was referenced for guidance https://khanhha.github.io/posts/Thin-Plate-Splines-Warping/
// But ultimately the source located here was ported: https://raw.githubusercontent.com/raphaelreme/tps/main/tps/thin_plate_spline.py

use faer::*;
use faer::prelude::*;

pub struct ThinPlateSpline {
	alpha: f32,
	parameters: Mat<f32>,
	control_points: Mat<f32>,
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

		let radial_distances = compute_radial_distances(source_mat, dest_mat);

		// Build K, X'_c, X'_c^T, and 0.
		// Build A from the above.
		// Build AP = Y and solve.
		let K = &radial_distances + (scale(alpha) * Mat::identity(n_c, n_c));
		let one_stack = ones(n_c, 1);
		let X_p = hstack(&one_stack.as_ref(), &source_mat.as_ref());
		let zeros = Mat::zeros(d_s + 1, d_s + 1);
		let A_top = hstack(&K, &X_p.as_ref());
		let A_bottom = hstack(&X_p.transpose(), &zeros);
		let A = vstack(&A_top.as_ref(), &A_bottom.as_ref());

		// Build Y from the destinations and some zero fills.
		let target_zeros = Mat::zeros(d_s + 1, dest_mat.ncols());
		let y = vstack(&destination_points.as_ref(), &target_zeros.as_ref());

		let parametes = solvers::Solver::
	}
}

fn ones(num_rows: usize, num_columns: usize) -> Mat<f32> {
	Mat::from_fn(num_rows, num_columns, |_| 1.0 )
}

fn hstack(left: &MatRef<f32>, right: &MatRef<f32>) -> Mat<f32> {
	assert_eq!(left.nrows(), right.nrows());
	let left_cols = left.ncols();
	Mat::from_fn(left.nrows(), left_cols + right.ncols(), |i, j| {
		if j < left_cols {
			left[(i, j)]
		} else {
			right[(i, j-left_cols)]
		}
	})
}

fn vstack(top: &MatRef<f32>, bottom: &MatRef<f32>) -> Mat<f32> {
	// I feel like there's a way to do this with transpose and hstack, but...
	assert_eq!(top.ncols(), bottom.ncols());
	let top_rows = top.nrows();
	Mat::from_fn(top.nrows() + bottom.nrows(), top.ncols(), |i, j| {
		if i < top_rows {
			top[(i, j)]
		} else {
			bottom[(i - top_rows, j)]
		}
	})
}

fn vec_to_mat(points: &Vec<f32>, num_rows: usize, num_columns: usize) -> Mat<f32> {
	Mat::from_fn(num_rows, num_columns, |i, j| {
		points[num_columns*i + j]
	})
}

fn compute_radial_distances(control: Mat<f32>, pts: Mat<f32>) -> Mat<f32> {
	todo!()
}

fn compute_pairwise_distances(a: &Mat<f32>, b: &Mat<f32>) -> Mat<f32> {
	assert_eq!(a.ncols(), b.ncols());
	Mat::from_fn(a.nrows(), b.nrows(), |i, j| {
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
	use super::*;

	#[test]
	fn test_distance() {
		let a = mat![
			[0.0f32, 0.0],
			[3.0, 4.0],
			[10.0, 10.0]
		];
		let origin = mat![
			[0.0f32, 0.0]
		];

		let distances = compute_pairwise_distances(&origin, &a);
		assert_eq!(distances[(0, 0)], 0.0);
		assert_eq!(distances[(0, 1)], 5.0);
		assert!((distances[(0, 2)] - 200.0_f32.sqrt()).abs() < 1e-5);
	}

	#[test]
	fn test_to_mat() {
		let values = vec![0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0];
		let expected = mat![
			[0.0f32, 1.0f32],
			[2.0, 3.0],
			[4.0, 5.0],
		];

		let out = vec_to_mat(&values, 3, 2);
		assert_matrix_eq!(expected, out);
	}
}