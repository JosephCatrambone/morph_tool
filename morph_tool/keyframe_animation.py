
import bisect
from dataclasses import dataclass, field
from typing import Optional, Tuple

import numpy

from common import lerp


@dataclass
class Keyframe:
	frame: int
	points_left: numpy.ndarray = field(default_factory=lambda: numpy.zeros((0, 2), dtype=float))
	points_right: numpy.ndarray = field(default_factory=lambda: numpy.zeros((0, 2), dtype=float))

	def add_point(self, a: Tuple[float, float], b: Tuple[float, float]):
		self.points_left = numpy.vstack((self.points_left, numpy.asarray(a, dtype=float)))
		self.points_right = numpy.vstack((self.points_right, numpy.asarray(b, dtype=float)))

	def update_point(self, a: Optional[Tuple[float, float]], b: Optional[Tuple[float, float]], idx: int):
		if a is not None:
			self.points_left[idx, :] = numpy.asarray(a, dtype=float)
		if b is not None:
			self.points_right[idx, :] = numpy.asarray(b, dtype=float)

	def remove_point(self, point_idx: int):
		self.points_left = numpy.delete(self.points_left, point_idx, axis=0)
		self.points_right = numpy.delete(self.points_right, point_idx, axis=0)

	@classmethod
	def interpolate(cls, a: 'Keyframe', b: 'Keyframe', amount: float):
		"""Linearly interpolate from a to b.  amount = 0 returns 'a'. amount = 1 returns 'b'."""
		return Keyframe(
			frame=int(lerp(a.frame, b.frame, amount)),
			points_left=lerp(a.points_left, b.points_left, amount),
			points_right=lerp(a.points_right, b.points_right, amount)
		)


class Animation:
	def __init__(self):
		self._keyframes = list()

	def _get_nearest_keyframe_idx(self, frame: int):
		"""
		Gets the index in _keyframe of the nearest keyframe, rounded up.
		For example, if we have keyframes at [0, 2, 10], a search for 0 will return 0.
		A search for 1 will return the index of '2' (1).
		A search for 3 will return the index of '10' (2).
		A search for 5 will return the index of '10' (2).
		A search for 10 will return the index of '10' (2).
		"""
		return bisect.bisect_left(self._keyframes, frame, key=lambda k: k.frame)

	def get_nearest_keyframe_before(self, frame: int):
		"""Return the keyframe BEFORE the current frame.  May return -1 if the keyframe is before the start."""
		return self._get_nearest_keyframe_idx(frame) - 1

	def get_nearest_keyframe_at_or_after(self, frame: int):
		return self._get_nearest_keyframe_idx(frame)

	def get_interpolated_frame(self, frame: int) -> Keyframe:
		# Special case: only one keyframe:
		if len(self._keyframes) < 2:
			return self._keyframes[0]
		current_or_next_idx = self.get_nearest_keyframe_at_or_after(frame)
		frame_before_idx = current_or_next_idx - 1  # TODO: assert frame is greater than zero?
		before = self._keyframes[frame_before_idx]
		after = self._keyframes[current_or_next_idx]  # 'after'.
		# Compute the interpolation amount by seeing where this is between the frames.
		amount = float(frame - before.frame) / float(after.frame - before.frame)
		# We're adding a keyframe for this point between two other keyframes, so we should LERP them and add this exact value.
		interpolated = Keyframe.interpolate(before, after, amount)
		return interpolated

	def add_point(self, left_match: Tuple[float, float], right_match: Tuple[float, float], frame: int):
		"""Add a correspondence points between two images at the given position.
		While this method is agnostic over y/x and x/y, """
		# TODO: This is basically a 2D data structure, like a spreadsheet or subdivided quad. Should we use something like Polars?
		# Is there a more clever way to represent this?

		# We need to go back through and make sure all the frames have the same number of points.
		# If this frame is between two other frames, we interpolate everything and add our new point.
		# Then we have to go back and add this point (at the given locations) to every other keyframe.
		insertion_idx = self.get_nearest_keyframe_at_or_after(frame)
		if len(self._keyframes) == 0:
			keyframe = Keyframe(frame)
			keyframe.add_point(left_match, right_match)
			self._keyframes.append(keyframe)
		else:
			# TODO: This won't work when we insert at the start or after the end.
			# We have to worry about interpolating inside.
			if self._keyframes[insertion_idx].frame == frame:
				# We _have_ a keyframe for this.  No need to interpolate.
				self._keyframes[insertion_idx].add_point(left_match, right_match)
			else:
				interpolated = self.get_interpolated_frame(frame)  # Yes, this should be frame and not insertion_idx.
				interpolated.add_point(left_match, right_match)
				self._keyframes.insert(insertion_idx, interpolated)

		# Go back and add this same point to every other keyframe.
		for idx, keyframe in enumerate(self._keyframes):
			if idx == insertion_idx:
				continue # Skip the one we already did.
			keyframe.add_point(left_match, right_match)

	def update_point(self, left_match: Tuple[float, float], right_match: Tuple[float, float], frame: int, idx: int):
		frame_idx = self._get_nearest_keyframe_idx(frame)
		assert self._keyframes[frame_idx].frame == frame, "Assertion Failed: Tried to update point at a keyframe which doesn't exist. Insert a point instead."
		self._keyframes[frame_idx].update_point(left_match, right_match, idx)

	def get_points(self, morph_amount: float, frame: int) -> Tuple[numpy.ndarray, numpy.ndarray, numpy.ndarray]:
		"""Return three numpy matrices, the 'left image' points, the 'right image' points, and the interpolated points.
		This will interpolate from the previous keyframe to the next keyframe based on 'frame' and from left image to
		right image based on morph amount."""
		frame_interp = self.get_interpolated_frame(frame)
		morph_interp = lerp(frame_interp.points_left, frame_interp.points_right, morph_amount)
		return frame_interp.points_left, frame_interp.points_right, morph_interp

	def remove_point(self, point_idx: int):
		for f in self._keyframes:
			f.remove_point(point_idx)
