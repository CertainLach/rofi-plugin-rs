#[macro_export]
macro_rules! emit {
	($action:expr) => {
		yield $action;
			()
	};
}

#[macro_export]
macro_rules! select {
    ($action:expr) => {
        match yield $action {
            UserAction::SelectItem(m, i) => (m, i),
            _ => return,
        }
    };
}

#[macro_export]
macro_rules! generator {
	($name:ident, $code:block) => {
		use $crate::{
			action::{ActionResult, GeneratorAction, UserAction},
			mode::Mode,
		};

		pub struct $name {
			message: Option<String>,
			items: Vec<String>,
			generator: Box<dyn Generator<UserAction, Return = (), Yield = GeneratorAction> + Unpin>,
		}

		impl Default for $name {
			fn default() -> Self {
				let mut generator = Box::new(move |_| $code);
				let state = Pin::new(&mut generator).resume(UserAction::EnterMenu);
				let mut items = Vec::new();
				let message;
				match state {
					GeneratorState::Yielded(action) => match action {
						GeneratorAction::ReplaceItems(new_message, new_items) => {
							message = new_message;
							items = new_items;
						}
						GeneratorAction::Default => message = None,
					},
					_ => message = Some("Plugin exited right after start".into()),
				};
				GeneratorRofi {
					message,
					items,
					generator,
				}
			}
		}

		impl Mode for $name {
			fn len(&self) -> usize {
				self.items.len()
			}

			fn name(&self, line: usize) -> &str {
				&self.items[line]
			}

			fn action(&mut self, action: UserAction) -> ActionResult {
				let action = Pin::new(&mut self.generator).resume(action);
				match action {
					GeneratorState::Yielded(GeneratorAction::ReplaceItems(msg, items)) => {
						self.message = msg;
						self.items = items;
						ActionResult::Reset
					}
					_ => ActionResult::Exit,
				}
			}

			fn message(&self) -> Option<&str> {
				self.message.as_ref().map(|f| f as &str)
			}
		}
	};
}
