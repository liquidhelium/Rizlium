//! Editing ui, basic guidelines: No mutable access to global
//! resources etc, but use response to emit events.

mod note;
mod spline;
mod timeline;
pub use note::note_editor_vertical;
pub use spline::*;
pub use timeline::*;
