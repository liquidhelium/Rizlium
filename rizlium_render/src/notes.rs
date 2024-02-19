use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{GameChart, GameChartCache, GameTime};

pub struct ChartNotePlugin;

impl Plugin for ChartNotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (add_notes,).run_if(resource_exists_and_changed::<GameChart>),
        )
        .add_systems(
            Update,
            (
                update_pos.run_if(chart_update!()),
                assocate_note.run_if(resource_exists_and_changed::<GameChart>),
            ),
        );
    }
}

// todo: convert to image
#[derive(Bundle)]
pub struct ChartNoteBundle {
    shape: ShapeBundle,
    note: ChartNote,
    stroke: Stroke,
}
impl Default for ChartNoteBundle {
    fn default() -> Self {
        Self {
            shape: ShapeBundle {
                path: GeometryBuilder::new()
                    .add(&Circle {
                        radius: 20.,
                        center: [0., 0.].into(),
                    })
                    .build(),
                ..default()
            },
            stroke: Stroke::new(Color::BLACK, 8.),
            note: default(),
        }
    }
}

#[derive(Component, Default)]
pub struct ChartNote;

#[derive(Component)]
pub struct ChartNoteId {
    pub line_idx: usize,
    pub note_idx: usize,
}

fn add_notes(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartNote>) {
    // info!("adding note");
    for _ in lines.iter().count()..chart.note_count() {
        // info!("adding note {}", i);
        commands.spawn(ChartNoteBundle::default());
    }
}

fn assocate_note(
    mut commands: Commands,
    chart: Res<GameChart>,
    notes: Query<Entity, With<ChartNote>>,
) {
    for (entity, (line_idx, note_idx)) in notes.iter().zip(chart.iter_note()) {
        // info!("assocating {line_idx}, {note_idx}");
        commands
            .entity(entity)
            .insert(ChartNoteId { line_idx, note_idx });
    }
}

fn update_pos(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    game_time: Res<GameTime>,
    mut notes: Query<(&mut Transform, &ChartNoteId)>,
) {
    notes
        .par_iter_mut()
        .for_each(|(mut transform, note_id)| {
            let time;
            {
                let Some(line) = chart.lines.get(note_id.line_idx) else {
                    return;
                };
                let Some(note) = line.notes.get(note_id.note_idx) else {
                    return;
                };
                time = note.time;
            }
            let chart_with_cache = chart.with_cache(&cache);
            let pos: Vec2 = chart_with_cache
                .line_pos_at_clamped(note_id.line_idx, time, **game_time)
                .unwrap()
                .into();
            *transform = transform.with_translation(pos.extend(0.));
        });
}
