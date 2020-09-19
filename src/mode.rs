use log::info;

use crate::action::{ActionResult, UserAction};

/// Rust native implementation of rofi_mode
pub trait Mode: Default {
	/// Returns count of results
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns readable name for line by number
	fn name(&self, line: usize) -> &str;

	/// Handles user actions
	fn action(&mut self, action: UserAction) -> ActionResult;

	/// Returns message to show bellow
	/// input field
	///
	/// Can contain pango markup (i.e `This is <b>bold</b> text`)
	fn message(&self) -> Option<&str>;

	/// Transforms user input, before giving them to
	/// matcher
	///
	/// Does nothing by default
	fn preprocess(&self, input: &str) -> String {
		info!("Input {}", input);
		input.to_owned()
	}
}
