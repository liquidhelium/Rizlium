use crate::prelude::{Note, Chart};

use crate::editing::{chart_path::NotePath, Result, ChartConflictError, commands::{ChartCommands, ChartCommand}};


pub struct ChangeNoteTime {
    pub(crate) modify_to: f32,
    pub(crate) note_path: NotePath,
}

impl ChartCommand for ChangeNoteTime {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let note = self.note_path.get_mut(chart)?;
        let current_time = note.time;
        note.time = self.modify_to;
        Ok(Self {
            modify_to: current_time,
            note_path: self.note_path,
        }
        .into())
    }
}

pub struct InsertNote {
    pub(crate) note: Note,
    pub(crate) note_path: NotePath,
}

impl ChartCommand for InsertNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self {
            note,
            note_path: NotePath(line_idx, note_idx),
        } = self;
        let notes = chart
            .lines
            .get_mut(line_idx)
            .and_then(|line| (note_idx <= line.notes.len()).then_some(&mut line.notes))
            .ok_or(ChartConflictError::InvalidNotePath {
                note_path: (line_idx, note_idx).into(),
            })?;
        notes.push(note);
        let len = notes.len();
        notes.swap(note_idx, len - 1);
        Ok(RemoveNote {
            note_path: self.note_path,
        }
        .into())
    }
}

pub struct RemoveNote {
    pub(crate) note_path: NotePath,
}

impl ChartCommand for RemoveNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self {
            note_path: NotePath(line_idx, note_idx),
        } = self;
        let note = chart
            .lines
            .get_mut(line_idx)
            .and_then(|line| {
                (note_idx < line.notes.len()).then(|| line.notes.swap_remove(note_idx))
            })
            .ok_or(ChartConflictError::InvalidNotePath {
                note_path: (line_idx, note_idx).into(),
            })?;
        Ok(InsertNote {
            note,
            note_path: self.note_path,
        }
        .into())
    }
}
#[cfg(test)]
mod test {
    use crate::editing::{test_resources::CHART, chart_path::NotePath, commands::ChartCommand};

    use super::{ChangeNoteTime, RemoveNote, InsertNote};
    #[test]
    fn change_time() {
        let mut chart = CHART.clone();
        let note_path = NotePath(7,0);
        let command = ChangeNoteTime {
            modify_to: 3.0,
            note_path,
        };
        let inversed = command.apply(&mut chart).unwrap();
        let note = note_path.get(&chart).unwrap();
        assert_eq!(note.time, 3.0);
        let com: ChangeNoteTime = inversed.try_into().unwrap();
        assert_eq!(com.modify_to, 4.0);
        com.apply(&mut chart).unwrap();
        let note = note_path.get(&chart).unwrap();
        assert_eq!(note.time, 4.0);
    }
    #[test]
    #[should_panic]
    fn invalid_note() {
        let path = NotePath(0,0);
        path.get(&CHART).unwrap();
    }

    #[test]
    fn insert_and_remove_note() {
        let mut chart = CHART.clone();
        let note_path = NotePath(7,0);
        let previous_len = chart.lines[7].notes.len();
        let insert: InsertNote = RemoveNote {
            note_path
        }.apply(&mut chart).unwrap().try_into().unwrap();
        println!("{:?}", insert.note);
        assert_eq!(chart.lines[7].notes.len(), previous_len - 1);
        let remove: RemoveNote = insert.apply(&mut chart).unwrap().try_into().unwrap();
        assert_eq!(remove.note_path, note_path); 
        assert_eq!(chart.lines[7].notes.len(), previous_len);

    }
}