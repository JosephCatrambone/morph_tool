
pub trait ImageSource {
	fn get_frame(self, frame_num: u32);
}