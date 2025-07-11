//! Midi to Rizlium Chart zip.

use std::{collections::HashMap, error, ffi::OsStr, fs, io, path::PathBuf};

use clap::Parser;
use midly::{Smf, Timing};
use mp3lame_encoder::Id3Tag;

mod convert;
mod midi_rendering;
mod packaging;

const DEFAULT_BACKGROUND: &[u8] = include_bytes!("assets/background.png");

#[derive(Parser, Debug)]
#[command(name = "Midi2Rzl", version, author)]
struct Args {
    /// 输入 MIDI 文件路径
    #[arg()]
    midi_path: PathBuf,
    /// 音源 SoundFont 或预渲染音频
    #[arg(short, long)]
    sound_source: PathBuf,
    /// 渲染 mp3 的采样率
    #[arg(long)]
    bitrate: Option<u32>,
    /// 背景图片路径
    #[arg(long)]
    background_file: Option<PathBuf>,
    /// 输出 zip 路径
    #[arg(short, long = "output")]
    output_path: Option<PathBuf>,
}

#[derive(Debug)]
struct Error(&'static str);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl error::Error for Error {}

fn main() {
    run(Args::parse()).unwrap_or_else(|e| {
        eprintln!("Something is wrong...I can feel it\n");
        eprintln!("Fault: \n {e:#?}");
        detailed_errmsg(e);
    })
}

fn detailed_errmsg(e: Box<dyn error::Error>) {
    if e.downcast_ref::<io::Error>().is_some() {
        eprintln!("..when tried to open the file")
    }
    if e.downcast_ref::<midly::Error>().is_some() {
        eprintln!("..when tried to read the midi file.")
    }
}

fn run(args: Args) -> Result<(), Box<dyn error::Error>> {
    let ProcessArgsResult {
        midi_path,
        output_path,
        sound_type,
        background_path,
        background_filename,
        sound_source,
        sample_rate,
        file_name,
    } = process_args(args)?;

    // 读取 MIDI 并生成谱面
    let file = fs::read(&midi_path)?;
    let smf = Smf::parse(&file)?;
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        _ => Err(midly::Error::new(&midly::ErrorKind::Invalid(
            "We support tick per beat times only.",
        )))?,
    };

    // 生成 Rizlium Chart
    let chart = convert::build_chart(&smf, ticks_per_beat);
    let chart_bytes = serde_json::to_vec(&chart)?;

    // 渲染或读取音乐
    let music = match sound_type {
        SoundType::SoundFont => {
            println!("rendering music...");
            let [left, right] =
                midi_rendering::render_midi(&sound_source, &midi_path, sample_rate)?;
            println!("encoding music...");
            midi_rendering::render_mp3(
                left,
                right,
                sample_rate,
                Id3Tag {
                    title: file_name.as_bytes(),
                    artist: b"midi2rzl",
                    album: &[],
                    album_art: &[],
                    year: &[],
                    comment: b"A rendered mp3 file, using LAME",
                },
            )
        }
        SoundType::PreRendered => {
            println!("reading pre-rendered music...");
            fs::read(&sound_source)?
        }
    };

    // 背景图片
    let mut background = background_path
        .as_os_str()
        .is_empty()
        .then(|| DEFAULT_BACKGROUND.to_owned());
    if background.is_none() {
        println!("reading background image...");
        background = Some(fs::read(&background_path)?);
    }
    // Info
    println!("generating info...");
    let info = format!(
        r#"name: {file_name}
format: Rizlium
chart_path: "{file_name}.json"
music_path: "{file_name}.mp3"
"#
    );
    let info_bytes = info.into_bytes();
    // 打包 zip
    println!("packing into zip..");
    let mut map = HashMap::with_capacity(3);
    map.insert(file_name.clone() + ".mp3", music);
    map.insert(file_name.clone() + ".json", chart_bytes);
    map.insert(background_filename, background.unwrap());
    map.insert("info.yml".to_string(), info_bytes);
    packaging::pack_files_into_zip(map, output_path)?;
    Ok(())
}

struct ProcessArgsResult {
    midi_path: PathBuf,
    output_path: PathBuf,
    sound_type: SoundType,
    background_path: PathBuf,
    background_filename: String,
    sound_source: PathBuf,
    sample_rate: u32,
    file_name: String,
}

enum SoundType {
    SoundFont,
    PreRendered,
}

fn process_args(args: Args) -> Result<ProcessArgsResult, Box<dyn error::Error>> {
    let Args {
        midi_path,
        sound_source,
        bitrate: sample_rate,
        background_file,
        output_path,
    } = args;
    let mut output_path = output_path.unwrap_or_default();
    let file_name = midi_path
        .file_stem()
        .unwrap_or(OsStr::new("result"))
        .to_string_lossy()
        .into_owned();
    let sound_extension = sound_source
        .extension()
        .ok_or(Error(
            "Can't indicate sound type.\nPlease make sure that the file has an extension.",
        ))?
        .to_str()
        .ok_or(Error("Sound file extension is not valid."))?;
    let sound_type = if sound_extension == "sf2" {
        SoundType::SoundFont
    } else {
        SoundType::PreRendered
    };
    if output_path.is_dir() || output_path.as_os_str().is_empty() {
        output_path.push(&file_name);
        output_path.set_extension("zip");
    }
    let background_path = background_file.unwrap_or_default();
    let background_filename = if background_path.as_os_str().is_empty() {
        "background.png".into()
    } else {
        background_path
            .file_name()
            .ok_or(Error("Can't get the file name."))?
            .to_string_lossy()
            .into_owned()
    };
    Ok(ProcessArgsResult {
        midi_path,
        output_path,
        background_path,
        background_filename,
        sound_source,
        sound_type,
        sample_rate: sample_rate.unwrap_or(44100),
        file_name,
    })
}
