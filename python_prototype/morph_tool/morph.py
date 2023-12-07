"""
Two variables: morph with TPS vs triangle deformation and static points vs animated points.
"""
import numpy
from PIL import Image

from tps import ThinPlateSpline

from common import color_lerp


def morph(
		left_points: numpy.ndarray,
		right_points: numpy.ndarray,
		left_image: Image.Image,
		right_image: Image.Image,
		morph_points: numpy.ndarray,
		pixel_blend: float,
		output_width: int,
		output_height: int
) -> Image.Image:
	# Images should be h, w, c.  This is the default if you do numpy.asarray(img).
	assert left_image.mode in {'L', 'RGB', 'RGBA'} and right_image.mode in {'L', 'RGB', 'RGBA'}
	# We find the mapping from our morph_points to the left image AND the mapping from our morph points to the right image.
	# Then for each output pixel we find the left image pixel and right image pixel and blend them.
	left_image = numpy.asarray(left_image)
	right_image = numpy.asarray(right_image)

	morph_to_left = ThinPlateSpline(alpha=0.1)
	morph_to_right = ThinPlateSpline(alpha=0.1)
	morph_to_left.fit(morph_points, left_points)
	morph_to_right.fit(morph_points, right_points)

	# Compute the sampling origin for each pixel.
	# Be wary: the mapping is made by providing (x,y) tuples. Our numpy arrays are using (y, x).
	output_coordinates = numpy.indices((output_width, output_height)).reshape((2, -1)).transpose() # Make a list of [[x, y], [x2, y2], ...]
	left_source_pixel = morph_to_left.transform(output_coordinates)
	left_y_coords = numpy.clip(left_source_pixel[:, 1], 0, left_image.shape[0]-1).astype(numpy.uint32) # Note: y is in index 1.
	left_x_coords = numpy.clip(left_source_pixel[:, 0], 0, left_image.shape[1]-1).astype(numpy.uint32)
	right_source_pixel = morph_to_right.transform(output_coordinates)
	right_y_coords = numpy.clip(right_source_pixel[:, 1], 0, right_image.shape[0]-1).astype(numpy.uint32) # Note: y is in index 1.
	right_x_coords = numpy.clip(right_source_pixel[:, 0], 0, right_image.shape[1]-1).astype(numpy.uint32)

	# Assemble the image
	out_image = numpy.zeros((output_height, output_width, left_image.shape[-1]), dtype=float)
	out_image[output_coordinates[:,1], output_coordinates[:,0], :] = (
		((1.0 - pixel_blend) * left_image[left_y_coords, left_x_coords, :]) +
		(pixel_blend * right_image[right_y_coords, right_x_coords, :])
	)
	return Image.fromarray(out_image.astype(numpy.uint8))
