use bevy::{platform::collections::HashMap, prelude::*, render::primitives::Aabb};
use bevy_prototype_lyon::{prelude::*, shapes::Circle};
use rizlium_chart::chart::NoteKind;

use crate::{colorrgba_to_color, hit_parcticles::HasHit, GameChart, GameChartCache, GameTime};

pub const NOTE_Z: f32 = 5.;

pub struct ChartNotePlugin;

impl Plugin for ChartNotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (add_notes,).run_if(resource_exists_and_changed::<GameChart>),
        )
        .add_systems(Update, assocate_note.run_if(resource_exists::<GameChart>))
        .add_systems(
            PostUpdate,
            (
                (update_note).run_if(chart_update!()),
                update_note_kind.run_if(resource_exists_and_changed::<GameChart>),
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
    transform: Transform,
    visibility: Visibility,
}

#[derive(Component, Default)]
#[require(CurrentNoteKind)]
pub struct ChartNote;

#[derive(Component)]
pub struct ChartNoteId {
    pub line_idx: usize,
    pub note_idx: usize,
}

#[derive(Component, Default)]
pub struct CurrentNoteKind(Option<NoteKind>);

fn add_notes(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartNote>) {
    // info!("adding note");
    for _ in lines.iter().count()..chart.note_count() {
        // info!("adding note {}", i);
        commands
            .spawn(ChartNoteBundle {
                note: default(),
                particle_time: HasHit(false),
                transform: Transform::from_translation(Vec2::ZERO.extend(NOTE_Z)),
                visibility: Visibility::Visible,
            })
            .insert(CurrentNoteKind::default());
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

fn update_note(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    game_time: Res<GameTime>,
    mut notes: Query<(&mut Transform, &ChartNoteId, &Children)>,
    mut note_bg: Query<&mut Sprite, With<note_tags::NoteBg>>,
) {
    notes
        .iter_mut()
        .for_each(|(mut transform, note_id, child)| {
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
            for child in child.iter() {
                if let Ok(mut sprite) = note_bg.get_mut(child) {
                    sprite.color = colorrgba_to_color(chart.theme_at(time).unwrap().this.color.note);
                }
            }
        });
}

macro_rules! tags {
    ($($a:ident),+) => {
        use bevy::prelude::*;
        $(
            #[derive(Component)]
            pub struct $a;
        )+
    };
}
mod note_tags {
    tags!(NoteFrame, NoteBg, Drag, HoldCap, HoldBody);
}

fn update_note_kind(
    mut commands: Commands,
    mut notes: Query<(
        Entity,
        &mut CurrentNoteKind,
        &ChartNoteId,
        Option<&Children>,
    )>,
    chart: Res<GameChart>,

    texture: Res<NoteTexture>,
) {
    let mut count = 0;
    if notes.is_empty() {
        warn!("No notes found to update.");
        return;
    }
    notes
        .iter_mut()
        .for_each(|(entity, mut kind, id, children)| {
            if let Some(line) = chart.lines.get(id.line_idx) {
                if let Some(note) = line.notes.get(id.note_idx) {
                    if kind.0.as_ref() == Some(&note.kind) {
                        return;
                    }
                    if let Some(children) = children {
                        commands.entity(entity).remove_children(children);
                    }
                    use note_tags::*;
                    match note.kind {
                        NoteKind::Tap => {
                            commands.spawn((
                                Sprite {
                                    image: texture.note_frame.clone(),
                                    custom_size: Some(Vec2::splat(78.)),
                                    ..default()
                                },
                                NoteFrame,
                                ChildOf(entity),
                            ));
                            commands.spawn((
                                Sprite {
                                    image: texture.note_bg.clone(),
                                    custom_size: Some(Vec2::splat(64.)),
                                    ..default()
                                },
                                Transform::from_translation(Vec2::ZERO.extend(-1.)),
                                NoteBg,
                                ChildOf(entity),
                            ));
                        }
                        NoteKind::Hold { .. } => {
                            commands.spawn((
                                Sprite {
                                    image: texture.hold_cap.clone(),
                                    custom_size: Some(Vec2::splat(64.)),
                                    ..default()
                                },
                                Transform::from_translation(Vec2::ZERO.extend(1.)),
                                HoldCap,
                                ChildOf(entity),
                            ));
                            commands.spawn((
                                Sprite {
                                    image: texture.hold_body.clone(),
                                    custom_size: Some(Vec2::splat(64.)),
                                    ..default()
                                },
                                Transform::from_translation(Vec2::ZERO.extend(0.)),
                                HoldBody,
                                ChildOf(entity),
                            ));
                        }
                        NoteKind::Drag => {
                            commands.spawn((
                                Sprite {
                                    image: texture.drag.clone(),
                                    custom_size: Some(Vec2::splat(64.)),
                                    ..default()
                                },
                                Transform::from_translation(Vec2::ZERO.extend(0.)),
                                Drag,
                                ChildOf(entity),
                            ));
                        }
                    }
                    kind.0 = Some(note.kind.clone());
                    count += 1;
                }
            }
        });
    if count > 0 {
        info!("Updated {count} notes.");
    }
}
