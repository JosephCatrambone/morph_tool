mod animation_system;
mod image_source;
mod thin_plate_spline;

struct ApplicationState {
	animation_system: animation_system::Animation,
	current_frame: u32,

}


pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
