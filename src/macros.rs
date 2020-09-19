use crate::{ffi::FfiMode, mode::Mode};

#[repr(transparent)]
pub struct RofiModeWrapper<T: Default + Mode>(pub FfiMode<T>);
unsafe impl<T: Default + Mode> Sync for RofiModeWrapper<T> {}

#[macro_export]
macro_rules! mode {
	($t:ty, $id:expr, $name:expr) => {
		use std::ptr::null_mut;
		use $crate::{ffi::FfiMode, macros::RofiModeWrapper};

		// Can't move this to const fn, because it will make impossible to handle cstrings
		#[no_mangle]
		pub static mut mode: RofiModeWrapper<$t> = RofiModeWrapper(FfiMode {
			abi_version: 6,
			name: $crate::real_c_string!($id),
			cfg_name_key: [0; 128],
			plugin_display_name: $crate::real_c_string!($name),

			init: FfiMode::init,
			destroy: FfiMode::destroy,

			num_entries: FfiMode::num_entries,
			result: FfiMode::result,
			token_match: FfiMode::token_match,
			display_name: FfiMode::display_name,

			get_completion: None,
			preprocess: FfiMode::preprocess,
			get_icon: None,
			message: FfiMode::message,

			private_data: None,
			free: FfiMode::free,
			ed: null_mut(),

			module: null_mut(),
		});
	};
}
