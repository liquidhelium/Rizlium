// 引入 Bevy 引擎的核心模块，特别是 f64 类型支持和预设。
use bevy::{math::f64, prelude::*};
// 引入 `bevy_kira_audio` 插件的核心组件，用于音频播放和控制。
use bevy_kira_audio::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, PlaybackState};

// 引入渲染模块中的 `GameChartCache` 和 `GameTime`。
use rizlium_render::{GameChartCache, GameTime};

/// `CurrentGameAudio` 是一个 Bevy 资源，用于存储当前正在播放的音频实例的句柄。
/// 通过这个句柄，我们可以控制音频的播放、暂停、seek 等操作。
#[derive(Resource, Debug)]
pub struct CurrentGameAudio(pub Handle<AudioInstance>);
/// `GameAudioSource` 是一个 Bevy 资源，用于存储已加载的音频源的句柄。
/// 当需要播放或重新播放音频时，会使用这个源。
#[derive(Resource, Deref)]
pub struct GameAudioSource(pub Handle<AudioSource>);

/// `TimeControlEvent` 是一个 Bevy 事件，用于从编辑器各处（如 UI、快捷键）发送时间控制命令。
#[derive(Event, Debug, Reflect)]
pub enum TimeControlEvent {
    Pause, // 暂停播放。
    Resume, // 恢复播放。
    Toggle, // 切换播放/暂停状态。
    Seek(f32), // 跳转到指定的时间点（秒）。
    SetPaused(bool), // 直接设置暂停状态。
    Advance(f32), // 将当前时间前进或后退指定的时长（秒）。
}

/// `TimeManager` 是一个 Bevy 资源，它独立于 Bevy 的 `Time` 资源，用于精确管理谱面的播放时间。
/// 它的核心思想是通过记录一个虚拟的“开始时间”和当前的“暂停时间点”来计算当前的播放进度。
#[derive(Resource, Debug, Default)]
pub struct TimeManager {
    // 记录了播放开始时对应的 `bevy::Time` 的绝对时间点。
    // 当 seek 或 resume 时，这个值会被调整，以确保 `current()` 的计算是正确的。
    start_time: f64,
    // 如果当前是暂停状态，则记录暂停时 `bevy::Time` 的绝对时间点。
    // 如果是 `None`，则表示正在播放。
    paused_since: Option<f64>,
    // 缓存当前帧的 `bevy::Time` 的绝对时间，由 `update_timemgr` 系统每帧更新。
    now: f64,
}

// `COMPENSATION_RATE` 是一个用于时间对齐的补偿系数。
// 当 `TimeManager` 的时间和音频的实际播放时间有微小偏差时，
// 它会以这个速率缓慢地将 `TimeManager` 的时间“拉”向音频时间，而不是立即跳变，以避免画面卡顿。
const COMPENSATION_RATE: f64 = 0.003;

/// `EditorAudioPlugin` 是负责集成所有时间和音频相关逻辑的 Bevy 插件。
pub struct EditorAudioPlugin;

impl Plugin for EditorAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin) // 添加 `bevy_kira_audio` 插件。
            .add_event::<TimeControlEvent>() // 注册时间控制事件。
            .add_systems(Startup, init_time_manager) // 在启动时初始化 `TimeManager`。
            .add_systems(
                Update,
                (
                    // `dispatch_events`：处理 `TimeControlEvent` 事件，仅在有音频实例时运行。
                    dispatch_events.run_if(resource_exists::<CurrentGameAudio>),
                    // `update_timemgr`：每帧更新 `TimeManager` 的 `now` 字段。
                    update_timemgr,
                    // `sync_audio`：当 `GameAudioSource` 改变时（即加载了新谱面），创建新的音频实例。
                    sync_audio.run_if(resource_exists_and_changed::<GameAudioSource>),
                    // `align_or_restart_audio`：核心系统，负责同步 `TimeManager` 和音频播放器的状态。
                    align_or_restart_audio.run_if(resource_exists::<CurrentGameAudio>),
                    // `game_time`：根据 `TimeManager` 的时间更新 `GameTime`，用于驱动渲染。
                    game_time.run_if(
                        resource_exists::<GameChartCache>.and(
                            resource_changed::<GameChartCache>
                                .or(resource_exists_and_changed::<TimeManager>),
                        ),
                    ),
                ),
            );
    }
}

/// `update_timemgr` 系统每帧运行，用于将 Bevy 的全局时间同步到 `TimeManager` 中。
fn update_timemgr(mut time: ResMut<TimeManager>, real_time: Res<Time>) {
    time.update(real_time.elapsed_secs_f64());
}

/// `dispatch_events` 系统处理所有传入的 `TimeControlEvent` 事件。
fn dispatch_events(
    mut event: EventReader<TimeControlEvent>,
    mut time: ResMut<TimeManager>,
    audio: Res<CurrentGameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
    audio_datas: Res<Assets<AudioSource>>,
    audio_data: Res<GameAudioSource>,
) {
    // 获取当前音频实例的可变引用。
    let Some(audio) = audios.get_mut(&audio.0) else {
        return;
    };
    // 获取音频源数据，用于获取音频总时长等信息。
    let Some(audio_data) = audio_datas.get(&**audio_data) else {
        warn!("invalid audio source");
        return;
    };
    // 遍历所有事件。
    for ev in event.read() {
        match ev {
            TimeControlEvent::Pause => time.pause(),
            TimeControlEvent::Resume => time.resume(),
            TimeControlEvent::Seek(pos) => {
                // 将 seek 位置限制在音频的有效范围内。
                let pos = (*pos as f64).clamp(0., audio_data.sound.duration().as_secs_f64() - 0.01);
                time.seek(pos);
                audio.seek_to(pos); // 同样需要 seek 音频实例。
            }
            TimeControlEvent::Toggle => time.toggle_paused(),
            TimeControlEvent::SetPaused(paused) => time.set_paused(*paused),
            TimeControlEvent::Advance(duration) => {
                // 计算并限制前进/后退的时长。
                let duration = (*duration as f64).clamp(
                    0.01 - time.current(),
                    audio_data.sound.duration().as_secs_f64() - 0.01 - time.current(),
                );
                time.advance(duration);
                audio.seek_by(duration); // 音频实例也相应地前进/后退。
            }
        }
    }
}

/// `sync_audio` 系统在加载新谱面（即 `GameAudioSource` 资源发生变化）时运行。
fn sync_audio(
    mut commands: Commands,
    game_audio: Option<ResMut<CurrentGameAudio>>,
    mut game_audios: ResMut<Assets<AudioInstance>>,
    mut time_control: EventWriter<TimeControlEvent>,
    source: Res<GameAudioSource>,
    audio: Res<Audio>,
) {
    // 使用新的音频源创建一个新的音频实例，并设置为循环、初始暂停。
    let new_current = audio.play(source.0.clone()).looped().paused().handle();
    if let Some(mut game_audio) = game_audio {
        // 如果之前已经有一个音频实例在播放，则停止它。
        if let Some(current) = game_audios.get_mut(&game_audio.0) {
            current.stop(default());
            // 更新 `CurrentGameAudio` 资源，指向新的音频实例。
            game_audio.0 = new_current;
        }
    } else {
        // 如果是第一次加载，则直接插入 `CurrentGameAudio` 资源。
        commands.insert_resource(CurrentGameAudio(new_current));
    }
    // 加载新音频后，强制进入暂停状态并 seek 到开头，以确保一个干净的起始状态。
    time_control.send(TimeControlEvent::Pause);
    time_control.send(TimeControlEvent::Seek(0.001));
}

/// `init_time_manager` 系统在应用启动时运行，用于创建和初始化 `TimeManager` 资源。
fn init_time_manager(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(TimeManager {
        start_time: time.elapsed_secs_f64(),
        paused_since: Some(time.elapsed_secs_f64()), // 初始状态为暂停。
        now: time.elapsed_secs_f64(),
    });
}

/// `game_time` 系统根据 `TimeManager` 的当前时间来更新 `GameTime` 资源。
/// `GameTime` 是渲染系统使用的最终时间，它可能经过了速度变化等处理。
fn game_time(cache: Res<GameChartCache>, time: Res<TimeManager>, mut game_time: ResMut<GameTime>) {
    // `cache.map_time` 会处理谱面中的变速事件。
    *game_time = GameTime(cache.map_time(time.current() as f32));
}

impl TimeManager {
    /// 每帧调用，用于更新当前时间。
    pub fn update(&mut self, now: f64) {
        self.now = now;
    }
    /// 获取虚拟的开始时间。
    pub fn start_time(&self) -> f64 {
        self.start_time
    }
    /// 切换暂停/播放状态。
    pub fn toggle_paused(&mut self) {
        info!("Toggling pause, current: {}", self.paused());
        if self.paused() {
            self.resume();
        } else {
            self.pause();
        }
    }
    /// 直接设置暂停状态。
    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            self.pause();
        } else {
            self.resume();
        }
    }
    /// 暂停时间管理器。
    pub fn pause(&mut self) {
        if self.paused() {
            return;
        }
        info!("pausing..");
        // 记录下当前时间作为暂停点。
        self.paused_since = Some(self.now);
    }
    /// 恢复时间管理器。
    pub fn resume(&mut self) {
        // `take()` 会移除 `paused_since` 中的值，留下 `None`。
        if let Some(paused) = self.paused_since.take() {
            info!("resuming");
            // 计算暂停前已经播放了多长时间。
            let delta = paused - self.start_time;
            // 根据暂停时长，重新计算一个新的虚拟 `start_time`，
            // 从而使得 `self.now - new_start_time` 能够延续暂停前的播放时间。
            let new_start = self.now - delta;
            self.start_time = new_start;
        }
    }
    /// 检查当前是否处于暂停状态。
    #[inline]
    pub fn paused(&self) -> bool {
        self.paused_since.is_some()
    }
    /// 跳转到指定的时间点。
    pub fn seek(&mut self, time: f64) {
        // 通过调整虚拟 `start_time` 来实现 seek。
        // `self.current()` 是 seek 前的时间，`time` 是目标时间。
        // `self.current() - time` 是需要调整的时间差。
        self.start_time += self.current() - time;
    }
    /// 获取当前的播放时间（秒）。
    pub fn current(&self) -> f64 {
        // 如果是暂停状态，则当前时间就是暂停点的时间。
        // 如果是播放状态，则用当前 Bevy 时间减去虚拟开始时间。
        self.paused_since.unwrap_or(self.now) - self.start_time
    }
    /// 将 `TimeManager` 的时间与音频播放器的实际时间进行对齐。
    pub fn align_to_audio_time(&mut self, audio_time: f64) {
        let current = self.current();
        // 如果时间差过大（超过10秒），则直接 seek，可能是由于卡顿或 seek 导致的。
        if (audio_time - current).abs() >= 10. {
            self.seek(audio_time);
            return;
        }
        // 如果时间差在可接受范围内，则使用 `COMPENSATION_RATE` 进行平滑补偿。
        self.seek((audio_time - current).mul_add(COMPENSATION_RATE, current));
    }
    /// 将时间前进或后退指定的时长。
    fn advance(&mut self, duration: f64) {
        self.seek(self.current() + duration);
    }
}

/// `align_or_restart_audio` 是核心的同步系统。
/// 它每帧检查 `TimeManager` 和 `bevy_kira_audio` 的状态，并确保它们保持一致。
fn align_or_restart_audio(
    mut time: ResMut<TimeManager>,
    mut audio: ResMut<CurrentGameAudio>,
    mut audios: ResMut<Assets<AudioInstance>>,
    player: Res<Audio>,
    source: Res<GameAudioSource>,
) {
    // 尝试获取当前的音频实例。
    let Some(current_audio) = audios.get_mut(&audio.0) else {
        // 如果句柄无效（例如，音频实例已被移除），则认为音频已停止，需要重启。
        info!("Restarting audio");
        // 创建一个新的音频实例。
        let new_handle = player.play(source.0.clone()).handle();
        audios.remove(&audio.0); // 移除旧的无效句柄。
        audio.0 = new_handle; // 更新资源。
        // 重置时间状态。
        time.seek(0.);
        time.pause();
        return;
    };

    // 根据音频实例的实际播放状态来决定如何同步。
    match current_audio.state() {
        // 如果音频正在播放...
        PlaybackState::Playing { position } => {
            // 但 `TimeManager` 却认为是暂停状态...
            if time.paused() {
                // ...则暂停音频。
                info!("Pausing audio");
                current_audio.pause(AudioTween::default());
            } else {
                // ...如果两者都认为在播放，则进行平滑的时间对齐。
                time.align_to_audio_time(position);
            }
        }
        // 如果音频不是正在播放状态（例如，已暂停或停止）...
        _ => {
            // ...则将音频 seek 到 `TimeManager` 的当前时间。
            current_audio.seek_to(time.current());
            // 并且，如果 `TimeManager` 认为应该在播放...
            if !time.paused() {
                // ...则恢复音频的播放。
                info!("Resuming audio");
                current_audio.resume(AudioTween::default());
            }
        }
    }
}