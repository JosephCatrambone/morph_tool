use anyhow::Result;
use std::path::Path;
use image::DynamicImage;

pub trait FrameProvider {
	fn get_frame(&mut self, frame_num: u32) -> &DynamicImage;
}

struct StaticImageProvider {
	img: DynamicImage,
}

impl StaticImageProvider {
	fn new_from_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
		let img = image::open(filename.as_ref())?;
		Ok(Self {
			img
		})
	}
}

impl FrameProvider for StaticImageProvider {
	fn get_frame(&mut self, _frame_num: u32) -> &DynamicImage {
		&self.img
	}
}

pub fn open_image_source() -> Option<Box<dyn FrameProvider>> {
	if let Some(fp) = rfd::FileDialog::new().pick_file() {
		// For now, we only support static file paths.
		todo!()
	} else {
		None
	}
}