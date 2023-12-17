use anyhow::Result;
use std::path::Path;
use image::{DynamicImage, GenericImage};

pub trait FrameProvider {
	fn get_frame(&mut self, frame_num: u32) -> &DynamicImage;
}

pub struct NullImageProvider {
	img: DynamicImage
}

impl NullImageProvider {
	pub fn new(size: Option<(u32, u32)>) -> Self {
		let (width, height) = size.unwrap_or((64, 64));
		let mut img = DynamicImage::new_rgba8(width, height);
		let block_size = 16;
		let y_blocks = height / block_size;
		let x_blocks = width / block_size;
		for y in 0..y_blocks {
			for x in 0..x_blocks {
				let fill = if (x+y) % 2 == 0 {
					[255, 0, 255, 255]
				} else {
					[255, 0, 255, 0]
				};
				for i in 0..block_size {
					for j in 0..block_size {
						img.put_pixel((x*block_size)+j, (y*block_size)+i, image::Rgba(fill));
					}
				}
			}
		}
		Self {
			img,
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