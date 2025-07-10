use std::time::Duration;

use bevy::{
    asset::{load_internal_asset, weak_handle},
    prelude::*,
};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_hanabi::*;

use crate::notes::ChartNoteId;

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
        app.add_systems(Startup, setup1)
            .add_systems(PreUpdate, spawn_particle_system);
    }
}

#[derive(Component)]
pub struct LatestParticleTime(Duration);
#[derive(Resource)]
pub struct ParticleTimer(Timer);

fn setup1(mut commands: Commands) {
    commands.insert_resource(ParticleTimer(Timer::new(
        Duration::from_secs(2),
        TimerMode::Repeating,
    )));
}

fn spawn_particle_system(
    mut commands: Commands,
    mut timer: ResMut<ParticleTimer>,
    time: Res<Time>,
    notes: Query<(&ChartNoteId, &LatestParticleTime)>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        info!("Spawning hit particles");
        commands.spawn((
            ParticleEffect {
                handle:BUILTIN_HIT_PARTICLE,
                prng_seed: Some(time.elapsed_secs().to_bits()),
            },
            Transform::from_translation(Vec3 {
                x: 0.,
                y: 0.,
                z: 10.,
            }),
        ));
    }
}
