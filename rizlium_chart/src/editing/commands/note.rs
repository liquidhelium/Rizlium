use crate::editing::chart_path::{ChartPath, LinePath};
use crate::prelude::{Chart, Note};

use crate::editing::{
    chart_path::NotePath,
    commands::{ChartCommand, ChartCommands},
    Result,
};

pub struct ChangeNoteTime {
    pub modify_to: f32,
    pub note_path: NotePath,
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
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
}

pub struct InsertNote {
    pub note: Note,
    pub line: LinePath,
    pub at: Option<usize>,
}

impl ChartCommand for InsertNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self { note, line, at } = self;
        let notes = &mut line.get_mut(chart)?.notes;
        let at_clamped = at.unwrap_or(notes.len()).clamp(0, notes.len());
        notes.insert(at_clamped, note);
        Ok(RemoveNote {
            note_path: NotePath(line, at_clamped),
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.line.valid(chart)
    }
}

pub struct RemoveNote {
    pub note_path: NotePath,
}

impl ChartCommand for RemoveNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self {
            note_path: NotePath(line_idx, note_idx),
        } = self;
        let note = self.note_path.remove(chart)?;

        Ok(InsertNote {
            note,
            line: line_idx,
            at: Some(note_idx),
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
}
#[cfg(test)]
mod test {
    use crate::editing::{
        chart_path::{ChartPath as _, NotePath},
        commands::ChartCommand,
        test_resources::CHART,
    };

    use super::{ChangeNoteTime, InsertNote, RemoveNote};
    #[test]
    fn change_time() {
        let mut chart = CHART.clone();
        let note_path = (7, 0).into();
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
        let path = NotePath::new(0, 0);
        path.get(&CHART).unwrap();
    }

    #[test]
    fn insert_and_remove_note() {
        let mut chart = CHART.clone();
        let note_path = NotePath::new(7, 0);
        let previous_len = chart.lines[7].notes.len();
        let insert: InsertNote = RemoveNote { note_path }
            .apply(&mut chart)
            .unwrap()
            .try_into()
            .unwrap();
        println!("{:?}", insert.note);
        assert_eq!(chart.lines[7].notes.len(), previous_len - 1);
        let remove: RemoveNote = insert.apply(&mut chart).unwrap().try_into().unwrap();
        assert_eq!(remove.note_path, note_path);
        assert_eq!(chart.lines[7].notes.len(), previous_len);
    }
}
