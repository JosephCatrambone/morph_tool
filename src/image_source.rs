use anyhow::Result;
use std::path::Path;
use image::DynamicImage;

const MISSING_ASSET_BYTES: &[u8] = include_bytes!("../resources/missing_asset.png");

pub trait FrameProvider {
	fn get_frame(&mut self, frame_num: u32) -> &DynamicImage;
}

pub struct NullImageProvider {
	img: DynamicImage
}

impl NullImageProvider {
	pub fn new() -> Self {
		Self {
			img: image::load_from_memory(MISSING_ASSET_BYTES).unwrap(),
		}
	}
}

impl FrameProvider for NullImageProvider {
	fn get_frame(&mut self, _frame_num: u32) -> &DynamicImage {
		&self.img
	}
}

pub struct StaticImageProvider {
	img: DynamicImage,
}

impl StaticImageProvider {
	pub fn new_from_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
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