use faer::Mat;

#[derive(Debug, Clone)]
struct Point(f32, f32);

impl Point {
	fn get_x(&self) -> f32 { self.0 }

	fn get_y(&self) -> f32 { self.1 }

	fn lerp(a: &Self, b: &Self, amount: f32) -> Self {
		Self {
			0: a.0 + (amount*(b.0 - a.0)),
			1: a.1 + (amount*(b.1 - a.1)),
		}
	}
}

impl From<(f32, f32)> for Point {
	fn from(value: (f32, f32)) -> Self {
		Point(value.0, value.1)
	}
}

impl From<Point> for (f32, f32) {
	fn from(value: Point) -> Self {
		(value.0, value.1)
	}
}

pub struct Keypoint {
	frame: u32,
	left: Point,
	right: Point,
}


pub struct Animation {
	// A single channel is a list of keypoints (point pairs) sorted by their frame.
	// A channel should not be empty. If it is, we should remove it and shift down the others.
	// Do not use swap_remove for channels.
	channels: Vec<Vec<Keypoint>>,
}

impl Animation {
	fn new() -> Self {
		Animation {
			channels: vec![]
		}
	}

	fn get_nearest_keyframe_idx(&self, frame: u32, channel: usize) -> usize {
		// If the channel is specified, find the nearest keyframe in that channel.  Otherwise search all channels.
		let nearest_kv = self.channels[channel].binary_search_by_key(&frame, |k| { k.frame } );

		// Even if the entry is NOT in the list, it returns where the value would be inserted, so this is >= the index.
		match nearest_kv {
			Ok(nearest) => nearest,
			Err(nearest) => nearest,
		}
	}

	/// Find a linear interpolation of this channel at the current frame.
	/// Will clamp to the 0th and last frames.
	fn interpolate_point(&self, frame: u32, channel: usize) -> (Point, Point) {
		let next_frame_idx = self.get_nearest_keyframe_idx(frame, channel);
		let previous_frame_idx = next_frame_idx.saturating_sub(1);
		let next_frame = self.channels[channel][next_frame_idx].frame;
		let previous_frame = self.channels[channel][previous_frame_idx].frame;
		if next_frame_idx == previous_frame_idx {
			(self.channels[channel][next_frame_idx].left.clone(), self.channels[channel][next_frame_idx].right.clone())
		} else {
			let amount = (frame as f32 - previous_frame as f32) / (next_frame as f32 - previous_frame as f32);
			let prev = &self.channels[channel][previous_frame_idx];
			let next = &self.channels[channel][next_frame_idx];
			let left_interp = Point::lerp(&prev.left, &next.left, amount);
			let right_interp = Point::lerp(&prev.right, &next.right, amount);
			(left_interp, right_interp)
		}
	}

	/// Insert a point pair at the given keyframe.
	/// If channel_idx is unspecified, this will create a new channel.
	/// If channel_idx is not None, a new keypoint will be added for the specified channel.
	/// The channel index of the insertion will be returned. You can think of this as 'point index'.
	///
	/// If a keyframe already exists for the specified (frame, channel) pair, this will replace it.
	pub fn set_point(&mut self, left_x: f32, left_y: f32, right_x: f32, right_y: f32, frame: u32, channel_idx: Option<usize>) -> usize {
		if let Some(c_idx) = channel_idx {
			let nearest_point_frame_idx = self.get_nearest_keyframe_idx(frame, c_idx);
			// This `nearest < len() check` helpfully handles the case where the channel is empty and where we would insert after the end.
			if nearest_point_frame_idx < self.channels[c_idx].len() && self.channels[c_idx][nearest_point_frame_idx].frame == frame {
				// If this 'nearest frame idx' matches the frame to insert, then we have to do a replacement.
				self.channels[c_idx][nearest_point_frame_idx].left = Point(left_x, left_y);
				self.channels[c_idx][nearest_point_frame_idx].right = Point(right_x, right_y);
				return c_idx;
			} else {
				// We need to add a new keypoint!
				self.channels[c_idx].insert(nearest_point_frame_idx, Keypoint {
					frame: frame,
					left: Point(left_x, left_y),
					right: Point(right_x, right_y),
				});
				return c_idx;
			}
		} else {
			// No channel specified, so we have to make a new channel.
			let new_channel_idx = self.channels.len();
			self.channels.push(vec![]);
			return self.set_point(left_x, left_y, right_x, right_y, frame, Some(new_channel_idx));
		}
	}

	/// Remove the given channel keypoint.
	/// If frame is None, will remove all keypoints from the channel and delete the channel.
	/// If a channel is deleted, remaining channel indices should be shifted down.
	/// If clear_point is called on a frame or channel that does not exist, this method will panic.
	pub fn clear_point(&mut self, frame: Option<u32>, channel_idx: usize) {
		if let Some(f) = frame {
			let nearest_keyframe_idx = self.get_nearest_keyframe_idx(f, channel_idx);
			if self.channels[channel_idx][nearest_keyframe_idx].frame != f {
				panic!("Attempted to remove keyframe {f} on channel {channel_idx} which does not exist.");
			}
			self.channels[channel_idx].swap_remove(nearest_keyframe_idx); // Okay to use swap_remove INSIDE channels.
		} else {
			self.channels.remove(channel_idx); // Can't swap-remove this, though.
		}
	}

	/// Return a tuple of left and right points, linearly interpolated by frame.
	/// Each vec contains [x, y, x, y, ...], a vector with 2*num channels elements.
	/// If frame is greater than the last frame, this method will panic.
	pub fn get_points(&self, frame: u32) -> (Vec<f32>, Vec<f32>) {
		let mut left_points = vec![];
		let mut right_points = vec![];

		for c_idx in 0..self.channels.len() {
			let (lp, rp) = self.interpolate_point(frame, c_idx);
			left_points.push(lp.get_x());
			left_points.push(lp.get_y());
			right_points.push(rp.get_x());
			right_points.push(rp.get_y());
		}

		(left_points, right_points)
	}

	pub fn get_num_channels(&self) -> usize {
		self.channels.len()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_point_sanity() {
		let pt_a = Point(0.0, 0.0);
		let pt_b = Point(2.0, 2.0);
		let interp = Point::lerp(&pt_a, &pt_b, 0.5);
		assert_eq!(interp.get_x(), 1.0);
		assert_eq!(interp.get_y(), 1.0);
	}

	#[test]
	fn basic_animation_sanity() {
		let mut anim = Animation::new();
		let channel = anim.set_point(0.0, 0.0, 0.0, 0.0, 0, None);
		anim.set_point(1.0, 0.0, 0.0, 0.0, 100, Some(channel));

		// There's one channel, so we expect 4 values (x, y, x, y) on each call to get_points.
		let mut pts = anim.get_points(0);
		// At frame zero we should match point 0.
		assert_eq!(pts.0, vec![0.0, 0.0]);
		assert_eq!(pts.1, vec![0.0, 0.0]);
		// At frame 50 we should be half way between our start and end.
		pts = anim.get_points(50);
		assert_eq!(pts.0, vec![0.5, 0.0]);
		assert_eq!(pts.1, vec![0.0, 0.0]);
		// At the end...
		pts = anim.get_points(100);
		assert_eq!(pts.0, vec![1.0, 0.0]);
		assert_eq!(pts.1, vec![0.0, 0.0]);
	}
}
