use bevy::{prelude::*, render::primitives::Aabb};
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{colorrgba_to_color, hit_parcticles::HasHit, GameChart, GameChartCache, GameTime};

pub const NOTE_Z: f32 = 5.;

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
                (update_pos).run_if(chart_update!()),
                assocate_note.run_if(resource_exists_and_changed::<GameChart>),
            ),
        );
    }
}

#[derive(Resource)]
pub struct NoteTexture {
    pub note_frame: Handle<Image>,
    pub note_bg: Handle<Image>,
    pub hold_body: Handle<Image>,
    pub hold_cap: Handle<Image>,
    pub drag: Handle<Image>,
}
// todo: convert to image
#[derive(Bundle)]
pub struct ChartNoteBundle {
    note: ChartNote,
    particle_time: HasHit,
    sprite: Sprite,
}

#[derive(Component, Default)]
pub struct ChartNote;

#[derive(Component)]
pub struct ChartNoteId {
    pub line_idx: usize,
    pub note_idx: usize,
}

fn add_notes(
    mut commands: Commands,
    chart: Res<GameChart>,
    lines: Query<&ChartNote>,
    texture: Res<NoteTexture>,
) {
    // info!("adding note");
    for _ in lines.iter().count()..chart.note_count() {
        // info!("adding note {}", i);
        commands
            .spawn(ChartNoteBundle {
                sprite: Sprite {
                    image: texture.note_frame.clone(),
                    custom_size: Some(Vec2::splat(78.)),
                    ..default()
                },
                note: default(),
                particle_time: HasHit(false),
            })
            .with_child((
                Sprite {
                    image: texture.note_bg.clone(),
                    custom_size: Some(Vec2::splat(64.)),
                    ..default()
                },
                Transform::from_translation(Vec2::ZERO.extend(-1.)),
            ));
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
    mut notes: Query<(&mut Transform, &ChartNoteId, &Children)>,
    mut sprites: Query<&mut Sprite>,
) {
    notes.iter_mut().for_each(|(mut transform, note_id, child)| {
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
        *transform = transform.with_translation(pos.extend(NOTE_Z));
        let Some(e)  = child.iter().next() else{
            warn!("No child found for note.");
            return;
        };
        let mut sprite = sprites.get_mut(e).unwrap_or_else(|_| {
            warn!("No sprite found for note.");
            panic!("No sprite found for note.");
        });
        sprite.color = colorrgba_to_color(chart.theme_at(time).unwrap().this.color.note);
    });
}
