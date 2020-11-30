use std::{os::raw::c_char, ptr::null_mut};

use crate::{ffi::FfiMode, mode::Mode};

/// Safe way to export Sync wrapper to rofi
#[repr(transparent)]
pub struct RofiModeWrapper<T: Default + Mode>(pub FfiMode<T>);
unsafe impl<T: Default + Mode> Sync for RofiModeWrapper<T> {}

pub const fn create_mode<M: Sized + Mode>(
	name: *const c_char,
	plugin_display_name: *const c_char,
) -> RofiModeWrapper<M> {
	RofiModeWrapper(FfiMode {
		abi_version: 6,
		name,
		cfg_name_key: [0; 128],
		plugin_display_name,

		init: FfiMode::init,
		destroy: FfiMode::destroy,

		num_entries: FfiMode::num_entries,
		result: FfiMode::result,
		token_match: FfiMode::token_match,
		display_name: FfiMode::display_name,

		get_completion: None,
		preprocess: FfiMode::preprocess,
		get_icon: FfiMode::get_icon,
		message: FfiMode::message,

		private_data: None,
		free: FfiMode::free,
		ed: null_mut(),

		module: null_mut(),
	})
}

#[macro_export]
macro_rules! mode {
	($t:ty, $id:expr, $name:expr) => {
		#[no_mangle]
		pub static mut mode: $crate::macros::RofiModeWrapper<$t> =
			$crate::macros::create_mode($crate::real_c_string!($id), $crate::real_c_string!($name));
	};
}
