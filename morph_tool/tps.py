# From https://raw.githubusercontent.com/raphaelreme/tps/main/tps/thin_plate_spline.py
# Maybe we should use https://github.com/cheind/py-thin-plate-spline/blob/master/thinplate/numpy.py ?
"""
	tps = ThinPlateSpline(0.5)
	tps.fit(output_ctrl, input_ctrl)

	# Create the 2d meshgrid of indices for output image
	output_indices = np.indices((height, width), dtype=np.float64).transpose(1, 2, 0)  # Shape: (H, W, 2)

	# Transform it into the input indices
	input_indices = tps.transform(output_indices.reshape(-1, 2)).reshape(height, width, 2)
"""

import numpy as np


# from scipy.spatial.distance import cdist  # type: ignore


class ThinPlateSpline:
	"""Solve the Thin Plate Spline interpolation

	Given a set of control points X_c \\ in R^{n_c \\times d_s} and target points X_t \\in R^{n_c \\times d_t}
	it learns a transformation f that maps X_c on X_t with some regularization.

	More formally:
	f = min_f E_{ext}(f) + \\alpha E_{int}(f)      (1)

	with E_{ext}(f) = \\sum_{i=1}^n ||X_{ti} - f(X_{ci})||_2^2
	and E_{int}(f) = \\iint \\left[\\left({\\frac{\\partial^2 f}{\\partial x_1^2}}\\right)^2
								+ 2\\left({\\frac{\\partial^2 f}{\\partial x_1\\partial x_2}}\\right)^2
								+  \\left({\\frac{\\partial^2 f}{\\partial x_2^2}}\\right)^2 \\right]{dx_1\\,dx_2


	Let X \\in R^{n \\times d_s} be n point from the source space. Then \\Phi(X) is the radial distance of those point
	to the control points \\in R^{n \\times n_c}:
	with d_{ij} = ||X_i - X_{cj}||_2, \\Phi(X)_{ij} = d_{ij}^2 \\log d_ij

	Then f(X) = A + X.B + \\Phi(X).C
	with A \\ in R^{d_t}, B \\in R^{d_s \\times d_t}, C \\ in R^{n_c \\times d_t} the parameters to learn.

	Learning A, B, C is done by solving a linear system so that f minimizes the energy (1) to transform X_c in X_t.

	The equation to solve is:

		   A      .   P   =   Y
						 <=>
	|  K   , X'_c|  | C |   |X_t|
	|            |  |   | = |   |
	|X'_c^T,   0 |  | B'|   | 0 |

	with X'_c = |1_{n_c}, X_c|  \\in R^{n_c \\times 1+d_s}, B'.T = |A, B.T|  \\in R^{d_t \\times 1+d_s}
	and K = \\Phi(X_c) + \\alpha I_{n_c}

	A \\in R^{(n_c + d_s + 1)\\times(n_c + d_s + 1)},
	P \\in R^{(n_c + d_s + 1)\\times d_t},
	Y \\in R^{(n_c + d_s + 1)\\times d_t},

	Attrs:
		alpha (float): Regularization parameter
		parameters (ndarray): All the parameters (P). Shape: (n_c + d_s + 1, d_t)
		control_points (ndarray): Control points fitted (X_c). Shape: (n_c, d_s)
	"""

	def __init__(self, alpha=0.0) -> None:
		self._fitted = False
		self.alpha = alpha

		self.parameters = np.array([], dtype=np.float32)
		self.control_points = np.array([], dtype=np.float32)

	def fit(self, source: np.ndarray, target: np.ndarray):
		"""Learn f that matches Y given X

		Args:
			source (ndarray): Control point at source space (X_c)
				Shape: (n_c, d_s)
			target (ndarray): Control point in the target space (X_t)
				Shape: (n_c, d_t)
		"""
		source = _ensure_2d(source)
		target = _ensure_2d(target)
		assert source.shape[0] == target.shape[0]

		n_c, d_s = source.shape
		self.control_points = source

		phi = self._radial_distance(source)

		# Build the linear system AP = Y
		X_p = np.hstack([np.ones((n_c, 1)), source])

		A = np.vstack(
			[np.hstack([phi + self.alpha * np.identity(n_c), X_p]), np.hstack([X_p.T, np.zeros((d_s + 1, d_s + 1))])]
		)

		target = np.vstack([target, np.zeros((d_s + 1, target.shape[1]))])

		self.parameters = np.linalg.solve(A, target)
		self._fitted = True

	def transform(self, source: np.ndarray) -> np.ndarray:
		"""Map source space to target space

		Args:
			source (ndarray): Points in the source space
				Shape: (n, d_s)

		Returns:
			ndarray: Mapped point in the target space
				Shape: (n, d_t)
		"""
		assert self._fitted, "Please call fit first."

		source = _ensure_2d(source)
		assert source.shape[1] == self.control_points.shape[1]

		phi = self._radial_distance(source)  # n x n_c

		source = np.hstack([phi, np.ones((source.shape[0], 1)), source])  # n x (n_c + 1 + d_s)
		return source @ self.parameters

	def _radial_distance(self, X: np.ndarray) -> np.ndarray:
		"""Compute the pairwise radial distances of the given points to the control points

		Input dimensions are not checked.

		Args:
			X (ndarray): N points in the source space
				Shape: (n, d_s)

		Returns:
			ndarray: The radial distance for each point to a control point (\\Phi(X))
				Shape: (n, n_c)
		"""
		dist = cdist(X, self.control_points)
		dist[dist == 0] = 1  # phi(r) = r^2 log(r) ->  (phi(0) = 0)
		return dist ** 2 * np.log(dist)


def _ensure_2d(array: np.ndarray) -> np.ndarray:
	"""Ensure that array is a 2d array

	In case of 1d array, let's expand the last dim
	"""
	assert array.ndim in (1, 2)

	# Expand last dim in order to interpret this as (n, 1) points
	if array.ndim == 1:
		array = array[:, None]

	return array


# Much slower than scipy...
def cdist(A: np.ndarray, B: np.ndarray) -> np.ndarray:
	"""Compute the pairwise euclidian distance between a set of points
	Do not check the dimensions of arguments
	Args:
		A (ndarray): n_a x d
		B (ndarray): n_b x d
	Returns:
		ndarray: n_a x n_b. D[i, j] = ||A[i] - B[j]||_2
	"""
	A = A[:, None, :]  # n_a x 1 x d
	B = B[None, :, :]  # 1 x n_b x d
	return np.linalg.norm(A - B, axis=2)  # n_a x n_b
