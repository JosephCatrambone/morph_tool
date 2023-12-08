use faer::Mat;

struct Point(f32, f32);

impl Point {
	fn get_x(&self) -> f32 { self.0 }

	fn get_y(&self) -> f32 { self.1 }

	fn lerp(a: Self, b: Self, amount: f32) -> Self {
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
	// Not a big fan. This is a list of keyframes across all channels. If channel 1 has a keyframe at 10 and channel 2 has a keyframe at 15, this will have [10, 15].
	keyframes: Vec<usize>,
	// A single channel is a list of keypoints (point pairs) sorted by their frame.
	// A channel should not be empty. If it is, we should remove it and shift down the others.
	// Do not use swap_remove for channels.
	channels: Vec<Vec<Keypoint>>,
}

impl Animation {
	fn get_nearest_keyframe_idx(&self, frame: u32, channel: Option<usize>) -> usize {
		// If the channel is specified, find the nearest keyframe in that channel.  Otherwise search all channels.
		let nearest_kv = if let Some(channel_idx) = channel {
			self.channels[channel_idx].binary_search_by_key(&frame, |k| { k.frame } )
		} else {
			self.keyframes.binary_search(frame.into())
		};

		// Even if the entry is NOT in the list, it returns where the value would be inserted, so this is >= the index.
		match nearest_kv {
			Ok(nearest) => nearest,
			Err(nearest) => nearest,
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
			let nearest_point_frame_idx = self.get_nearest_keyframe_idx(frame, Some(c_idx));
			// If this 'nearest frame idx' matches the frame to insert, then we have to do a replacement.
			if self.channels[c_idx][nearest_point_frame_idx].frame == frame {
				// Replacement!
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
				// And update our keyframes appropriately.
				let keyframe_idx = self.keyframes.binary_search(frame.into()).expect("Couldn't find keyframe insertion point?");
				self.keyframes.insert(keyframe_idx, frame as usize);
				return c_idx;
			}
		} else {
			// No channel specified, so we have to make a new channel.
			let new_channel_idx = self.channels.len();
			todo!()
		}
	}

	pub fn clear_point(&mut self, frame: u32, channel_idx: usize) {
		// Remove the given channel (point) from every keyframe.
		todo!()
	}

	pub fn get_points(&self, frame: u32) -> Vec<f32> {
		// Interpolate all points between the two given frames.
		todo!()
	}

	pub fn get_num_channels(&self) -> usize {
		self.channels.len()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
	}
}
