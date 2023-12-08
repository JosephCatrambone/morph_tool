use polars::prelude::*;

pub struct Animation {
	data: DataFrame
}

impl Animation {
	fn new() -> Self {
		let dataframe = df!(
			"keyframe" => Vec::<u32>::new(),
			"channel" => Vec::<u32>::new(),
			"x_left" => Vec::<f32>::new(),
			"y_left" => Vec::<f32>::new(),
			"x_right" => Vec::<f32>::new(),
			"y_right" => Vec::<f32>::new(),
		).unwrap();

		Self {
			data: dataframe
		}
	}

	/// Insert a point pair at the given keyframe.
	/// If channel_idx is unspecified, this will create a new channel.
	/// If channel_idx is not None, a new keypoint will be added for the specified channel.
	/// The channel index of the insertion will be returned. You can think of this as 'point index'.
	///
	/// If a keyframe already exists for the specified (frame, channel) pair, this will replace it.
	pub fn set_point(&mut self, left_x: f32, left_y: f32, right_x: f32, right_y: f32, frame: u32, channel_idx: Option<usize>) -> usize {
		// See if the given point at the given channel exists and remove it if so.
		self.data.clone().lazy().select()
		todo!()
	}

	pub fn clear_point(&mut self, frame: u32, channel_idx: usize) {
		// Remove the given channel (point) from every keyframe.
		todo!()
	}

	pub fn get_points(&self, frame: u32) -> Vec<f32> {
		// Interpolate all points between the two given frames.
		todo!()
	}

	pub fn get_num_keyframes(&self) -> usize {
		self.data.clone().lazy().select([col("keyframe")]).collect()?.height()
	}

	pub fn get_num_channels(&self) -> usize {
		self.data.clone().lazy().select([col("channel")]).unique(None, UniqueKeepStrategy::Any).collect()?.height()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
	}
}
