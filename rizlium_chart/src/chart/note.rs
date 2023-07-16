#[derive(Debug, Clone)]
pub enum NoteKind {
    Tap,
    Hold { end: f32 },
    Drag,
}
#[derive(Debug, Clone)]
pub struct Note {
    pub time: f32,
    pub kind: NoteKind,
}

impl Note {
    pub fn new(time: f32, kind: NoteKind) -> Self {
        Self { time, kind }
    }
}
