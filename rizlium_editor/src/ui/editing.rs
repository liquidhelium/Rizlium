//! Editing ui, basic guidelines: No mutable access to global 
//! resources etc, but use response to emit events.

mod spline;
mod note;
mod timeline;
pub use spline::*;
pub use note::note_editor_vertical;
pub use timeline::*;