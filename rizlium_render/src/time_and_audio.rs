use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, PlaybackState};
use std::ops::Deref;

use crate::chart::GameChartCache;

#[derive(Resource, Debug)]
pub struct CurrentGameAudio(pub Handle<AudioInstance>);
#[derive(Resource)]
pub struct GameAudioSource(pub Handle<AudioSource>);

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
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .init_resource::<GameTime>()
            .add_event::<TimeControlEvent>()
            .add_systems(Startup, init_time_manager)
            .add_systems(
                Update,
                (
                    dispatch_events,
                    update_timemgr,
                    sync_audio.run_if(resource_exists_and_changed::<GameAudioSource>()),
                    align_audio.run_if(resource_exists::<CurrentGameAudio>()),
                    game_time.run_if(resource_exists::<GameChartCache>()),
                ),
            );
    }
}
#[derive(Event)]
pub enum TimeControlEvent {
    Pause,
    Resume,
    Toggle,
    Seek(f32),
    SetPaused(bool),
}

fn update_timemgr(mut time: ResMut<TimeManager>, real_time: Res<Time>) {
    time.update(real_time.raw_elapsed_seconds());
}

fn dispatch_events(mut event: EventReader<TimeControlEvent>, mut time: ResMut<TimeManager>) {
    for ev in event.iter() {
        match ev {
            TimeControlEvent::Pause => time.pause(),
            TimeControlEvent::Resume => time.resume(),
            TimeControlEvent::Seek(pos) => time.seek(*pos),
            TimeControlEvent::Toggle => time.toggle_paused(),
            TimeControlEvent::SetPaused(paused) => time.set_paused(*paused),
        }
    }
}

fn sync_audio(
    mut commands: Commands,
    game_audio: Option<ResMut<CurrentGameAudio>>,
    mut game_audios: ResMut<Assets<AudioInstance>>,
    mut time_control: EventWriter<TimeControlEvent>,
    source: Res<GameAudioSource>,
    audio: Res<Audio>,
) {
    let new_current = audio.play(source.0.clone()).handle();
    info!("Syncing audio...");
    if let Some(mut game_audio) = game_audio {
        if let Some(current) = game_audios.get_mut(&game_audio.0) {
            current.stop(default());
            game_audio.0 = new_current;
        }
    } else {
        commands.insert_resource(CurrentGameAudio(new_current));
    }
    time_control.send(TimeControlEvent::Pause);
    time_control.send(TimeControlEvent::Seek(0.1));
}

fn init_time_manager(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(TimeManager {
        start_time: time.raw_elapsed_seconds(),
        paused_since: None,
        now: time.raw_elapsed_seconds(),
    });
}
fn game_time(cache: Res<GameChartCache>, time: Res<TimeManager>, mut game_time: ResMut<GameTime>) {
    if time.paused() {
        return;
    }
    let since_start = time.current();
    *game_time = GameTime(
        cache
            .0
            .beat
            .value_padding(since_start)
            .expect("cache is empty"),
    );
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
        info!("Toggling pause, current: {}", self.paused());
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
        info!("pausing..");
        self.paused_since = Some(self.now);
    }
    pub fn resume(&mut self) {
        if let Some(paused) = self.paused_since.take() {
            info!("resuming");
            let delta = paused - self.start_time;
            let new_start = self.now - delta;
            self.start_time = new_start;
        }
    }
    #[inline]
    pub fn paused(&self) -> bool {
        self.paused_since.is_some()
    }
    pub fn seek(&mut self, time: f32) {
        self.start_time += self.current() - time;
    }
    pub fn current(&self) -> f32 {
        self.paused_since.unwrap_or(self.now) - self.start_time
    }
    pub fn align_to_audio_time(&mut self, audio_time: f32) {
        let current = self.current();
        self.seek((audio_time - current).mul_add(COMPENSATION_RATE, current));
    }
}

fn align_audio(
    mut time: ResMut<TimeManager>,
    audio: Res<CurrentGameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
) {
    let Some(audio) = audios.get_mut(&audio.0) else {
        return;
    };

    if let PlaybackState::Playing { position } = audio.state() {
        if time.paused() {
            info!("Pausing audio");
            audio.pause(AudioTween::default());
        } else {
            time.align_to_audio_time(position as f32);
        }
    } else {
        audio.seek_to(time.current().into());
        if !time.paused() {
            info!("resuming audio");
            audio.resume(AudioTween::default());
        }
    }
}
