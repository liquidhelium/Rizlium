use bevy::{
    asset::{load_internal_asset, weak_handle},
    prelude::*,
};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_hanabi::*;

use crate::{notes::ChartNoteId, GameChart, GameChartCache, GameTime};

pub struct HitParticlePlugin;

pub const BUILTIN_HIT_PARTICLE: Handle<EffectAsset> =
    weak_handle!("99ae43c6-fcb3-49ce-8c2a-44f7cef9aff6");

impl Plugin for HitParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<EffectAsset>::new(&[".json"]));
        load_internal_asset!(
            app,
            BUILTIN_HIT_PARTICLE,
            "../assets/particle.json",
            |a, _| serde_json::from_str(a).expect("Bad effect asset")
        );
        app.init_resource::<QueuedHitParticles>()
            .add_systems(Startup, set_up_spawner)
            .add_systems(
                Update,
                (spawn_particle_system, spawn_queued_particles).run_if(chart_update!()),
            );
    }
}

fn set_up_spawner(mut commands: Commands) {
    commands.spawn((
        ParticleEffect {
            handle: BUILTIN_HIT_PARTICLE,
            prng_seed: None,
        },
        Transform::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 40.,
        }),
    ));
}

#[derive(Component)]
pub struct HasHit(pub(crate) bool);

#[derive(Resource, Default)]
struct QueuedHitParticles {
    particles: Vec<Vec2>,
}

fn spawn_particle_system(
    time: Res<GameTime>,
    mut notes: Query<(&ChartNoteId, &mut HasHit)>,
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    mut queued_particles: ResMut<QueuedHitParticles>,
) {
    notes.iter_mut().for_each(|(id, mut has_hit)| {
        let Some(note) = chart
            .lines
            .get(id.line_idx)
            .and_then(|line| line.notes.get(id.note_idx))
        else {
            return;
        };
        if note.time < **time && !has_hit.0 {
            let pos = chart
                .with_cache(&cache)
                .line_pos_at_clamped(id.line_idx, note.time, note.time)
                .unwrap();
            // info!("Queueing hit particles");
            queued_particles.particles.push(vec2(pos[0], pos[1]));
            has_hit.0 = true;
        } else if note.time > **time + 0.05 {
            has_hit.0 = false;
        }
    });
}

fn spawn_queued_particles(
    time: Res<GameTime>,
    mut spawner: Query<(&mut EffectSpawner, &mut Transform, &mut ParticleEffect)>,
    mut queued_particles: ResMut<QueuedHitParticles>,
) {
    if queued_particles.particles.is_empty() {
        return;
    }
    let Some((mut spawner, mut transform, mut particle)) = spawner.single_mut().ok() else {
        return;
    };
    // only one spawn is allow in a frame
    if let Some(pos) = queued_particles.particles.pop() {
        // info!("Spawning hit particles");
        particle.prng_seed = Some(time.to_bits());
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        spawner.reset();
    }
}
