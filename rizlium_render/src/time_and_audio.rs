use bevy::prelude::*;

use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, PlaybackState};
use std::ops::Deref;

use crate::chart::GameChartCache;

#[derive(Resource, Debug)]
pub struct CurrentGameAudio(pub Handle<AudioInstance>);
#[derive(Resource, Deref)]
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

const COMPENSATION_RATE: f32 = 0.003;

#[derive(Resource, Debug, Default)]
pub struct TimeManager {
    start_time: f32,
    paused_since: Option<f32>,
    now: f32,
}

pub struct TimeAndAudioPlugin {
    pub manual_time_control: bool,
}

#[derive(Resource, Default)]
pub struct ManualGameTime(f32);

impl Plugin for TimeAndAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .init_resource::<GameTime>();

        if !self.manual_time_control {
            app.add_event::<TimeControlEvent>()
                .add_systems(Startup, init_time_manager)
                .add_systems(
                    Update,
                    (
                        dispatch_events.run_if(resource_exists::<CurrentGameAudio>),
                        update_timemgr,
                        sync_audio.run_if(resource_exists_and_changed::<GameAudioSource>),
                        align_or_restart_audio.run_if(resource_exists::<CurrentGameAudio>),
                        game_time.run_if(
                            resource_exists::<GameChartCache>.and_then(
                                resource_changed::<GameChartCache>
                                    .or_else(resource_exists_and_changed::<TimeManager>),
                            ),
                        ),
                    ),
                );
        } else {
            app.init_resource::<ManualGameTime>()
                .add_systems(PreUpdate, update_gametime_manual);
        }
    }
}
fn update_gametime_manual(
    cache: Res<GameChartCache>,
    time: Res<ManualGameTime>,
    mut game_time: ResMut<GameTime>,
) {
    *game_time = GameTime(cache.map_time(time.0));
}
#[derive(Event, Debug, Reflect)]
pub enum TimeControlEvent {
    Pause,
    Resume,
    Toggle,
    Seek(f32),
    SetPaused(bool),
    Advance(f32),
}

fn update_timemgr(mut time: ResMut<TimeManager>, real_time: Res<Time>) {
    time.update(real_time.elapsed_secs());
}

fn dispatch_events(
    mut event: EventReader<TimeControlEvent>,
    mut time: ResMut<TimeManager>,
    audio: Res<CurrentGameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
    audio_datas: Res<Assets<AudioSource>>,
    audio_data: Res<GameAudioSource>,
) {
    let Some(audio) = audios.get_mut(&audio.0) else {
        return;
    };
    let Some(audio_data) = audio_datas.get(&**audio_data) else {
        warn!("invalid audio source");
        return;
    };
    for ev in event.read() {
        match ev {
            TimeControlEvent::Pause => time.pause(),
            TimeControlEvent::Resume => time.resume(),
            TimeControlEvent::Seek(pos) => {
                let pos = pos.clamp(0., audio_data.sound.duration().as_secs_f32() - 0.01);
                time.seek(pos);
                audio.seek_to(pos.into());
            }
            TimeControlEvent::Toggle => time.toggle_paused(),
            TimeControlEvent::SetPaused(paused) => time.set_paused(*paused),
            TimeControlEvent::Advance(duration) => {
                let duration = duration.clamp(
                    0.01 - time.current(),
                    audio_data.sound.duration().as_secs_f32() - 0.01 - time.current(),
                );
                time.advance(duration);audio.seek_by(duration.into());
            }
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
    let new_current = audio.play(source.0.clone()).looped().handle();
    if let Some(mut game_audio) = game_audio {
        if let Some(current) = game_audios.get_mut(&game_audio.0) {
            current.stop(default());
            game_audio.0 = new_current;
        }
    } else {
        commands.insert_resource(CurrentGameAudio(new_current));
    }
    time_control.send(TimeControlEvent::Pause);
    time_control.send(TimeControlEvent::Seek(0.01));
}

fn init_time_manager(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(TimeManager {
        start_time: time.elapsed_secs(),
        paused_since: Some(time.elapsed_secs()),
        now: time.elapsed_secs(),
    });
}
fn game_time(cache: Res<GameChartCache>, time: Res<TimeManager>, mut game_time: ResMut<GameTime>) {
    *game_time = GameTime(cache.map_time(time.current()));
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
        if (audio_time - current).abs() >= 10. {
            self.seek(audio_time);
            return;
        }
        self.seek((audio_time - current).mul_add(COMPENSATION_RATE, current));
    }
    /// Advance [`TimeManager`] by `duration`.
    /// `duration` can be negative to rewind.
    fn advance(&mut self, duration: f32) {
        self.seek(self.current() + duration);
    }
}

fn align_or_restart_audio(
    mut time: ResMut<TimeManager>,
    mut audio: ResMut<CurrentGameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
    player: Res<Audio>,
    source: Res<GameAudioSource>,
) {
    let Some(current_audio) = audios.get_mut(&audio.0) else {
        info!("Restarting audio");
        let new_handle = player.play(source.0.clone()).handle();
        audios.remove(&audio.0);
        audio.0 = new_handle;
        time.seek(0.);
        time.pause();
        return;
    };

    match current_audio.state() {
        PlaybackState::Playing { position } => {
            if time.paused() {
                info!("Pausing audio");
                current_audio.pause(AudioTween::default());
            } else {
                time.align_to_audio_time(position as f32);
            }
        }
        _ => {
            current_audio.seek_to(time.current().into());
            if !time.paused() {
                info!("Resuming audio");
                current_audio.resume(AudioTween::default());
            }
        }
    }
}
