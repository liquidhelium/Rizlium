use std::time::Duration;

use bevy::{asset::load_internal_asset, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_hanabi::*;

use crate::notes::ChartNoteId;

pub struct HitParticlePlugin;

pub const BUILTIN_HIT_PARTICLE: Handle<EffectAsset> = Handle::weak_from_u128(11451419198103301);

impl Plugin for HitParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<EffectAsset>::new(&[".json"]));
        load_internal_asset!(
            app,
            BUILTIN_HIT_PARTICLE,
            "../assets/particle.json",
            |a, _| serde_json::from_str(a).expect("Bad effect asset")
        );
    }
}

#[derive(Component)]
pub struct LatestParticleTime(Duration);

fn spawn_particle_system(
    mut commands: Commands,
    notes: Query<(&ChartNoteId, &LatestParticleTime)>,
) {
    commands.spawn(ParticleEffectBundle {
        effect: ParticleEffect::new(BUILTIN_HIT_PARTICLE).with_z_layer_2d(Some(30.)),
        ..default()
    });
}
