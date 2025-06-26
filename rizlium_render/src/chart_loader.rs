use std::{
    borrow::Cow,
    io::{Cursor, Read},
};

use bevy::{
    prelude::{ResMut, *},
    tasks::{IoTaskPool, Task},
};
use bevy_kira_audio::{prelude::StaticSoundData, AudioSource};
use rizlium_chart::prelude::{Chart, RizlineChart};
use serde::Deserialize;
use snafu::{ResultExt, Snafu};
use zip::ZipArchive;

use crate::{GameAudioSource, GameChart};

pub struct ChartLoadingPlugin;

impl Plugin for ChartLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadChartEvent>()
            .add_event::<ChartLoadingEvent>()
            .init_resource::<PendingChart>()
            .add_systems(
                PostUpdate,
                (
                    dispatch_load_event,
                    (unpack_chart).run_if(|r: Res<PendingChart>| r.0.is_some()),
                ),
            );
    }
}
#[derive(Deserialize, Default)]
pub struct ChartInfo {
    pub name: String,
    pub format: ChartFormat,
    pub chart_path: String,
    pub music_path: String,
}

#[derive(Deserialize, Default)]
pub enum ChartFormat {
    #[default]
    Rizline,
    Rizlium,
}

#[derive(Event)]
pub struct LoadChartEvent(pub String);

#[derive(Event)]
pub enum ChartLoadingEvent {
    Success(String),
    Error(ChartLoadingError),
}

impl ChartLoadingEvent {
    fn err(err: ChartLoadingError) -> Self {
        Self::Error(err)
    }
}

pub struct BundledGameChart {
    music: AudioSource,
    chart: Chart,
    path: String,
    // todo: handle chart info
    _info: ChartInfo,
}

#[derive(Resource, Default)]
pub struct PendingChart(Option<Task<Result<BundledGameChart, ChartLoadingError>>>);

#[derive(Snafu, Debug)]
pub enum ChartLoadingError {
    #[snafu(display("Failed to read file: {}", source))]
    UnzipFileFailed {
        source: zip::result::ZipError,
    },
    #[snafu(display("Failed to read file: {}", source))]
    ReadingFileFailed {
        source: std::io::Error,
    },
    #[snafu(display("No file named {} in the zip archive", file_name))]
    NoFileInZip {
        file_name: Cow<'static, str>,
        source: zip::result::ZipError,
    },
    #[snafu(display("Chart format is invalid: {}", source))]
    ChartFormatInvalid {
        source: serde_json::Error,
    },
    #[snafu(display("Chart info format is invalid: {}", source))]
    InfoFormatInvalid {
        source: serde_yaml::Error,
    },
    #[snafu(display("Failed to convert chart: {}", source))]
    ChartConvertingFailed {
        source: rizlium_chart::parse::ConvertError,
    },
    #[snafu(display("Failed to convert music: {}", source))]
    MusicConvertingFailed {
        source: kira::sound::FromFileError,
    },
}

fn load_chart(path: String, mut pending: ResMut<PendingChart>) {
    let r: Task<Result<BundledGameChart, _>> = IoTaskPool::get().spawn(async {
        let mut file = async_fs::read(path.clone())
            .await
            .context(ReadingFileFailedSnafu)?;
        let mut res =
            ZipArchive::new(Cursor::new(file.as_mut_slice())).context(UnzipFileFailedSnafu)?;
        let info_file = res.by_name("info.yml").context(NoFileInZipSnafu {
            file_name: "info.yml",
        })?;
        let info: ChartInfo = serde_yaml::from_reader(info_file).context(InfoFormatInvalidSnafu)?;
        let chart_path = &info.chart_path;
        let music_path = &info.music_path;
        let chart: Chart = match info.format {
            ChartFormat::Rizline => {
                let chart: RizlineChart =
                    serde_json::from_reader(res.by_name(chart_path).context(NoFileInZipSnafu {
                        file_name: chart_path.clone(),
                    })?)
                    .context(ChartFormatInvalidSnafu)?;
                chart.try_into().context(ChartConvertingFailedSnafu)?
            }
            ChartFormat::Rizlium => {
                serde_json::from_reader(res.by_name(chart_path).context(NoFileInZipSnafu {
                    file_name: chart_path.clone(),
                })?)
                .context(ChartFormatInvalidSnafu)?
            }
        };
        let mut sound_data = Vec::new();
        res.by_name(music_path)
            .context(NoFileInZipSnafu {
                file_name: music_path.clone(),
            })?
            .read_to_end(&mut sound_data)
            .context(ReadingFileFailedSnafu)?;
        let music = bevy_kira_audio::AudioSource {
            sound: StaticSoundData::from_cursor(Cursor::new(sound_data))
                .context(MusicConvertingFailedSnafu)?,
        };
        Ok(BundledGameChart {
            music,
            chart,
            path,
            _info: info,
        })
    });
    pending.0 = Some(r);
}

fn dispatch_load_event(
    mut events: EventReader<LoadChartEvent>,
    pending_chart: ResMut<PendingChart>,
) {
    if events.len() > 1 {
        warn!("Mutiple charts are requested, ignoring previous ones.");
    }

    {
        let Some(event) = events.read().last() else {
            return;
        };
        if pending_chart.0.is_some() {
            warn!("Replacing previous unloaded chart.");
        }
        info!("Loading chart {}...", &event.0);
        load_chart(event.0.clone(), pending_chart);
    }
    events.clear();
}

fn unpack_chart(
    mut pending_chart: ResMut<PendingChart>,
    mut commands: Commands,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut ev: EventWriter<ChartLoadingEvent>,
) {
    let Some(chart) = pending_chart
        .0
        .as_mut()
        .and_then(|c| futures_lite::future::block_on(futures_lite::future::poll_once(c)))
    else {
        return;
    };
    pending_chart.0 = None;
    match chart {
        Err(err) => {
            ev.write(ChartLoadingEvent::err(err));
        }
        Ok(bundle) => {
            commands.insert_resource(GameChart::new(bundle.chart));
            let audio_handle = audio_sources.add(bundle.music);
            commands.insert_resource(GameAudioSource(audio_handle));
            info!("completed loading chart");
            ev.write(ChartLoadingEvent::Success(bundle.path));
        }
    }
}
