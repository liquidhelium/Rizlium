use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioTween, PlaybackState};
use std::ops::Deref;

use crate::chart::GameChartCache;

#[derive(Resource, Debug)] 
pub struct GameAudio(pub Handle<AudioInstance>);
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct GameTime(f32);
impl Deref for GameTime {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


const COMPENSATION_RATE: f32 = 0.001;

#[derive(Resource, Debug, Default)]
pub struct TimeManager {
    start_time: f32,
    paused_since: Option<f32>,
    now: f32,
}

pub struct TimeAndAudioPlugin;

impl Plugin for TimeAndAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .init_resource::<GameTime>()
            .add_systems(Startup, (audio, init_time_manager))
            .add_systems(Update, (align_audio, game_time.run_if(resource_exists::<GameChartCache>())));
    }
}
fn audio(mut commands: Commands,server: Res<AssetServer>, audio: Res<Audio>) {
    let handle =  audio.play(server.load("/home/helium/code/rizlium/rizlium_render/assets/take.ogg")).handle();
    commands.insert_resource(GameAudio(handle));
}

fn init_time_manager(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(TimeManager {
        start_time: time.raw_elapsed_seconds(),
        paused_since: None,
        now: time.raw_elapsed_seconds(),
    });
}
fn game_time(cache: Res<GameChartCache>, time: Res<TimeManager>, mut game_time: ResMut<GameTime>) {
    // todo: start
    let since_start = time.current();
    *game_time = GameTime(cache.0.beat.value_padding(since_start).expect("cache is empty"));
}

impl TimeManager {
    /// 每帧都要同步此时间
    pub fn update(&mut self, now: f32) {
        self.now = now;
    }
    pub fn start_time(&self) -> f32 {
        self.start_time
    }
    pub fn toggle_paused(&mut self) {
        if self.paused() {
            self.resume();
        } else {
            self.pause();
        }
    }
    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            self.pause();
        } else {
            self.resume();
        }
    }
    pub fn pause(&mut self) {
        if self.paused() {
            return;
        }
        self.paused_since = Some(self.now);
    }
    pub fn resume(&mut self) {
        if let Some(paused) = self.paused_since.take() {
            let delta = paused - self.start_time;
            let new_start = self.now - delta;
            self.start_time = new_start;
        }
    }
    pub fn paused(&self) -> bool {
        self.paused_since.is_some()
    }
    pub fn seek(&mut self, time: f32) {
        self.start_time -= time;
    }
    pub fn current(&self) -> f32 {
        self.paused_since.unwrap_or(self.now) - self.start_time
    }
    pub fn align_to_audio_time(&mut self, audio_time: f32) {
        self.seek((audio_time - self.current()) * COMPENSATION_RATE)
    }
}

fn align_audio(
    mut time: ResMut<TimeManager>,
    real_time: Res<Time>,
    audio: Res<GameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
) {
    let Some(audio) = audios.get_mut(&audio.0) else {
        return;
    };
    time.update(real_time.raw_elapsed_seconds());

    match audio.state() {
        PlaybackState::Playing { position } => {
            time.align_to_audio_time(position as f32);
            if time.paused() {
                audio.pause(AudioTween::default());
            }
        }
        _ => {
            audio.seek_to(time.current().into());
            if !time.paused() {
                audio.resume(AudioTween::default());
            }
        }
    }
}
