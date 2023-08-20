//! Editing ui, basic guidelines: No mutable access to global 
//! resources etc, but use response to emit events.

mod spline;
mod note;
pub use spline::*;