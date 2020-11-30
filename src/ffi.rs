use std::{
	cell::Ref, cell::RefCell, cell::RefMut, convert::TryInto, ffi::CStr, ffi::CString,
	os::raw::c_char, ptr::null,
};

use crate::{
	action::SelectData,
	action::{ActionResult, UserAction},
	mode::Icon,
	mode::Mode,
};

extern "C" {
	fn helper_token_match(tokens: *const RofiIntMatcher, input: *const c_char) -> i32;
	fn rofi_icon_fetcher_query(name: *const c_char, size: i32) -> u32;
	fn rofi_icon_fetcher_get(uid: u32) -> *const ();
}

#[repr(i32)]
pub enum ModeMode {
	/// Exit.
	ModeExit = 1000,
	/// Skip to the next cycle-able dialog.
	NextDialog = 1001,
	/// Reload current DIALOG
	ReloadDialog = 1002,
	/// Previous dialog
	PreviousDialog = 1003,
	/// Reloads the dialog and unset user input
	ResetDialog = 1004,
}

#[repr(i32)]
pub enum MenuReturn {
	/// Entry is selected.
	Ok = 0x00010000,
	/// User canceled the operation. (e.g. pressed escape)
	Cancelled = 0x00020000,

	/// User requested a mode switch
	Next = 0x00040000,
	/// Go to the previous menu.
	Previous = 0x00400000,

	/// Custom (non-matched) input was entered.
	CustomInput = 0x00080000,
	/// User wanted to delete entry from history.
	EntryDelete = 0x00100000,
	/// User wants to jump to another switcher.
	QuickSwitch = 0x00200000,

	/// User wants to jump to custom command, i.e +Ctrl
	CustomCommand = 0x00800000,
	/// Bindings specifics, i.e +Shift
	CustomAction = 0x10000000,

	/// Mask
	LowerMask = 0x0000FFFF,
}

#[repr(i32)]
pub enum State {
	Default = 0,
	Warning = 1,
	Error = 2,
	Default2 = 3,
}

#[repr(C)]
pub struct RofiIntMatcher {
	regex: usize,
	invert: bool,
}

#[repr(C)]
pub struct FfiMode<T: Default + Mode> {
	pub abi_version: i32,

	pub name: *const c_char,
	pub cfg_name_key: [u8; 128],
	pub plugin_display_name: *const c_char,

	pub init: extern "C" fn(&mut Self) -> i32,
	pub destroy: extern "C" fn(&mut Self),

	pub num_entries: extern "C" fn(&Self) -> u32,
	pub result: extern "C" fn(&Self, i32, *const *const c_char, u32) -> ModeMode,
	pub token_match: unsafe extern "C" fn(&Self, *const RofiIntMatcher, u32) -> i32,
	pub display_name: extern "C" fn(&Self, u32, &mut i32, usize, i32) -> *const c_char,
	pub get_icon: extern "C" fn(&Self, u32, i32) -> *const (),
	pub get_completion: Option<extern "C" fn(&Self, u32) -> *mut c_char>,
	pub preprocess: unsafe extern "C" fn(&Self, *const c_char) -> *const c_char,
	pub message: extern "C" fn(&Self) -> *const c_char,

	pub private_data: Option<Box<RefCell<T>>>,
	pub free: extern "C" fn(&mut Self),
	pub ed: *mut u8,

	pub module: *mut u8,
}

impl<T: Mode + Default> FfiMode<T> {
	fn data(&self) -> Ref<T> {
		Option::as_ref(&self.private_data).expect("some").borrow()
	}
	fn data_mut(&self) -> RefMut<T> {
		Option::as_ref(&self.private_data)
			.expect("some")
			.borrow_mut()
	}

	pub extern "C" fn init(&mut self) -> i32 {
		env_logger::init();
		log::info!("Hello, world!");
		self.private_data
			.replace(Box::new(RefCell::new(T::default())));
		1
	}

	pub extern "C" fn num_entries(&self) -> u32 {
		self.data().len().try_into().expect("u32 limit")
	}
	pub extern "C" fn get_icon(&self, line: u32, size: i32) -> *const () {
		let data = self.data();
		let icon = data.icon(line as usize, size as u32);
		match icon {
			Some(Icon::Named(name)) => {
				let uid = unsafe {
					rofi_icon_fetcher_query(CString::new(&name as &str).unwrap().into_raw(), size)
				};
				unsafe { rofi_icon_fetcher_get(uid) }
			}
			None => null(),
		}
	}
	pub extern "C" fn display_name(
		&self,
		selected_line: u32,
		state: &mut i32,
		_attr_list: usize,
		_get_entry: i32,
	) -> *const c_char {
		*state = State::Default as i32;
		CString::new(self.data().name(selected_line as usize))
			.expect("utf8")
			.into_raw()
	}
	pub extern "C" fn result(
		&self,
		mretv: i32,
		_input: *const *const c_char,
		selected_line: u32,
	) -> ModeMode {
		if (mretv & MenuReturn::Ok as i32) != 0 {
			match self.data_mut().action(UserAction::SelectItem(
				SelectData {
					ctrl: (mretv & MenuReturn::CustomCommand as i32) != 0,
					shift: (mretv & MenuReturn::CustomAction as i32) != 0,
				},
				selected_line as usize,
			)) {
				ActionResult::Reset => ModeMode::ResetDialog,
				ActionResult::Exit => ModeMode::ModeExit,
			}
		} else if (mretv & MenuReturn::Next as i32) != 0 {
			ModeMode::NextDialog
		} else if (mretv & MenuReturn::Previous as i32) != 0 {
			ModeMode::PreviousDialog
		} else {
			ModeMode::ModeExit
		}
	}

	pub extern "C" fn message(&self) -> *const c_char {
		if let Some(msg) = self.data().message() {
			CString::new(msg).unwrap().into_raw()
		} else {
			null()
		}
	}

	pub extern "C" fn destroy(&mut self) {
		self.private_data.take();
	}

	// TODO: expose to `Mode`
	/// # Safety
	/// Called by rofi
	pub unsafe extern "C" fn token_match(&self, tokens: *const RofiIntMatcher, index: u32) -> i32 {
		helper_token_match(
			tokens,
			CString::new(self.data().name(index as usize))
				.unwrap()
				.into_raw(),
		)
	}

	/// # Safety
	/// Called by rofi
	pub unsafe extern "C" fn preprocess(&self, input: *const c_char) -> *const c_char {
		let input = CStr::from_ptr(input).to_str().unwrap();
		let out = self.data().preprocess(input);
		CString::new(out).unwrap().into_raw()
	}

	pub extern "C" fn free(&mut self) {}
}
