use std::io::{Cursor, Read};

use bevy::{
    asset::{AssetLoader, LoadState, LoadedAsset},
    prelude::*,
    prelude::{AssetServer, Assets, ResMut},
    reflect::{TypePath, TypeUuid},
};
use bevy_kira_audio::{
    prelude::{StaticSoundData, StaticSoundSettings},
    AudioSource,
};
use rizlium_chart::prelude::{Chart, RizlineChart};
use serde::Deserialize;
use zip::ZipArchive;

use crate::{GameAudioSource, GameChart};

pub struct ChartLoadingPlugin;

impl Plugin for ChartLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GameChartAsset>()
            .add_asset_loader(GameChartLoader)
            .add_event::<LoadChartEvent>()
            .add_systems(
                PostUpdate,
                (
                    chart_loader_system,
                    (unpack_loaded_chart, remove_failure_and_report)
                        .run_if(resource_exists::<PendingGameChartHandle>()),
                ),
            );
    }
}

#[derive(TypeUuid, TypePath)]
#[uuid = "d935131a-18f3-429d-9821-65ec60a2a025"]
pub struct GameChartAsset {
    music: AudioSource,
    chart: Chart,
    info: ChartInfo,
}

#[derive(Default)]
pub struct GameChartLoader;

impl AssetLoader for GameChartLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            #[cfg(feature = "trace")]
            let span = info_span!("load chart");
            #[cfg(feature = "trace")]
            let _enter = span.enter();
            //     let Some(suffix) = load_context.path().extension() else {
            //     return Ok(())
            // };
            //     let suffix = suffix.to_str().expect("invalid path");
            let reader = Cursor::new(bytes);
            // match suffix {
            //     ".zip" => {
            let mut res = ZipArchive::new(reader)?;
            #[cfg(feature = "trace")]
            let span = info_span!("load info");
            #[cfg(feature = "trace")]
            let _enter = span.enter();
            let info_file = res.by_name("info.yml")?;
            let info: ChartInfo = serde_yaml::from_reader(info_file)?;
            #[cfg(feature = "trace")]
            drop(_enter);
            #[cfg(feature = "trace")]
            let span = info_span!("load chart it self");
            #[cfg(feature = "trace")]
            let _enter = span.enter();
            let chart_path = &info.chart_path;
            let music_path = &info.music_path;
            #[cfg(feature = "trace")]
            let span = info_span!("Deserialize chart");
            #[cfg(feature = "trace")]
            let _enter1 = span.enter();
            let chart: RizlineChart = serde_yaml::from_reader(res.by_name(chart_path)?)?;
            #[cfg(feature = "trace")]
            drop(_enter1);
            #[cfg(feature = "trace")]
            let span = info_span!("Convert chart");
            #[cfg(feature = "trace")]
            let _enter1 = span.enter();
            let chart: Chart = chart.try_into()?;
            #[cfg(feature = "trace")]
            drop(_enter1);
            #[cfg(feature = "trace")]
            drop(_enter);
            #[cfg(feature = "trace")]
            let span = info_span!("load music");
            #[cfg(feature = "trace")]
            let _enter = span.enter();
            #[cfg(feature = "trace")]
            let span = info_span!("extract music");
            #[cfg(feature = "trace")]
            let _enter1 = span.enter();
            let mut sound_data = Vec::new();
            res.by_name(music_path)?.read_to_end(&mut sound_data)?;
            #[cfg(feature = "trace")]
            drop(_enter1);
            #[cfg(feature = "trace")]
            let span = info_span!("create music");
            #[cfg(feature = "trace")]
            let _enter1 = span.enter();
            let music = bevy_kira_audio::AudioSource {
                sound: StaticSoundData::from_cursor(
                    Cursor::new(sound_data),
                    StaticSoundSettings::default(),
                )?,
            };
            #[cfg(feature = "trace")]
            drop(_enter1);
            #[cfg(feature = "trace")]
            drop(_enter);
            load_context.set_default_asset(LoadedAsset::new(GameChartAsset { music, chart, info }));

            //     }
            //     _ => unreachable!("Bevy should guarantee the extension"),
            // };
            Ok(())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["zip"]
    }
}

#[derive(Deserialize, Default)]
struct ChartInfo {
    pub name: String,
    pub format: ChartFormat,
    pub chart_path: String,
    pub music_path: String,
}

#[derive(Deserialize, Default)]
enum ChartFormat {
    #[default]
    Rizline,
}

#[derive(Event)]
pub struct LoadChartEvent(pub String);

#[derive(Resource)]
struct PendingGameChartHandle {
    handle: Handle<GameChartAsset>,
}

impl PendingGameChartHandle {
    fn new(handle: Handle<GameChartAsset>) -> Self {
        Self { handle }
    }
}

fn chart_loader_system(
    mut commands: Commands,
    mut events: EventReader<LoadChartEvent>,
    pending: Option<Res<PendingGameChartHandle>>,
    asset_server: Res<AssetServer>,
) {
    if events.len() > 1 {
        warn!("Mutiple charts are requested, ignoring previous ones.");
    }

    {
        let Some(event) = events.iter().last() else {
            return;
        };
        if pending.is_some() {
            warn!("Replacing previous unloaded chart.");
        }
        info!("Loading chart {}...", &event.0);
        let handle: Handle<GameChartAsset> = asset_server.load(&event.0);
        commands.insert_resource(PendingGameChartHandle::new(handle.clone()));
    }
    events.clear();
}

fn unpack_loaded_chart(
    mut ev: EventReader<AssetEvent<GameChartAsset>>,
    mut commands: Commands,
    pending: Res<PendingGameChartHandle>,
    mut chart_assets: ResMut<Assets<GameChartAsset>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    let Some(AssetEvent::Created { handle } )=ev.iter().next() else {
        return;
    };
    let pending_handle = &pending.handle;
    if pending_handle != handle {
        info!("Ignoring loaded chart {:?}.", handle);
        return;
    }
    // check loading
    commands.remove_resource::<PendingGameChartHandle>();
    let asset = chart_assets
        .remove(pending_handle)
        .expect("Pending handle does not exist");
    commands.insert_resource(GameChart::new(asset.chart));
    let audio_handle = audio_sources.add(asset.music);
    commands.insert_resource(GameAudioSource(audio_handle));
    info!("Completed loading chart")
}

fn remove_failure_and_report(
    mut commands: Commands,
    pending: Res<PendingGameChartHandle>,
    server: Res<AssetServer>,
) {
    if server.get_load_state(&pending.handle) == LoadState::Failed {
        error!("Loading chart failed");
        commands.remove_resource::<PendingGameChartHandle>();
    }
}
