#![feature(
	generators,
	generator_trait,
	const_fn,
	const_fn_fn_ptr_basics,
	const_mut_refs
)]
pub use real_c_string::real_c_string;

pub mod action;
pub mod ffi;
pub mod macros;
pub mod mode;

pub mod generator;
pub mod list;
