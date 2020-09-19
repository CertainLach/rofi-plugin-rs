pub struct SelectData {
	pub ctrl: bool,
	pub shift: bool,
}

pub enum UserAction {
	EnterMenu,
	SelectItem(SelectData, usize),
}

pub enum GeneratorAction {
	Default,
	ReplaceItems(Option<String>, Vec<String>),
}

pub enum ActionResult {
	Reset,
	Exit,
}
