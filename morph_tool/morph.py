"""
Two variables: morph with TPS vs triangle deformation and static points vs animated points.
"""
import numpy

from tps import ThinPlateSpline

from common import lerp


def morph(
		left_points: numpy.ndarray,
		right_points: numpy.ndarray,
		left_image: numpy.ndarray,
		right_image: numpy.ndarray,
		morph_points: numpy.ndarray,
		pixel_blend: float,
		output_width: int,
		output_height: int
) -> numpy.ndarray:
	# Images should be h, w, c.  This is the default if you do numpy.asarray(img).
	assert left_image.shape[-1] in {1, 3, 4} and right_image.shape[-1] in {1, 3, 4}, "Error: input images are not channels-last."
	# Might want to change the signature here.
	# We have to do some two-way binding here.
	# We find the mapping from our morph_points to the left image AND the mapping from our morph points to the right image.
	# Then for each output pixel we find the left image pixel and right image pixel and blend them.
	morph_to_left = ThinPlateSpline(alpha=0.1)
	morph_to_right = ThinPlateSpline(alpha=0.1)
	morph_to_left.fit(morph_points, left_points)
	morph_to_right.fit(morph_points, right_points)
	out_image = numpy.zeros((output_height, output_width, left_image.shape[-1]), dtype=float)
	for y in range(0, output_height):
		for x in range(0, output_width):
			left_pixel_pos = morph_to_left.transform(numpy.asarray([[x, y]]))
			right_pixel_pos = morph_to_right.transform(numpy.asarray([[x, y]]))
			left_pixel_value = left_image[int(left_pixel_pos[1]), int(left_pixel_pos[0]), :]  # TODO: Bilinear interpolate.
			right_pixel_value = right_image[int(right_pixel_pos[1]), int(right_pixel_pos[0]), :]
			out_pixel_value = lerp(left_pixel_value, right_pixel_value, pixel_blend)
			out_image[y, x, :] = out_pixel_value
	return out_image
