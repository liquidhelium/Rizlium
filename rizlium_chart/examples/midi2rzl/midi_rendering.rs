use mp3lame_encoder::Id3Tag;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::{error, fs::File, path::Path, sync::Arc};

pub fn render_midi<P1, P2>(
    sound_font: P1,
    midi: P2,
    sample_rate: u32,
) -> Result<[Vec<f32>; 2], Box<dyn error::Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Load the SoundFont.
    let mut sf2 = File::open(sound_font)?;
    let sound_font = Arc::new(SoundFont::new(&mut sf2)?);

    // Load the MIDI file.
    let mut mid = File::open(midi)?;
    let midi_file = Arc::new(MidiFile::new(&mut mid)?);

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(sample_rate as i32);
    let synthesizer = Synthesizer::new(&sound_font, &settings)?;
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file.
    sequencer.play(&midi_file, false);

    // The output buffer.
    let sample_count = (settings.sample_rate as f64 * midi_file.get_length()) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    sequencer.render(&mut left[..], &mut right[..]);
    Ok([left, right])
}

pub fn render_mp3(left: Vec<f32>, right: Vec<f32>, sample_rate: u32, id3_tag: Id3Tag) -> Vec<u8> {
    use mp3lame_encoder::{Builder, DualPcm, FlushNoGap};

    let mut mp3_encoder = Builder::new().expect("Create LAME builder");
    mp3_encoder.set_num_channels(2).expect("set channels");
    mp3_encoder
        .set_sample_rate(sample_rate)
        .expect("set sample rate");
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps192)
        .expect("set brate");
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .expect("set quality");
    mp3_encoder.set_id3_tag(id3_tag).expect("set ID3 tag");
    let mut mp3_encoder = mp3_encoder.build().expect("To initialize LAME encoder");

    //use actual PCM data
    let input = DualPcm {
        left: left.as_slice(),
        right: right.as_slice(),
    };

    let mut mp3_out_buffer =
        Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(input.left.len()));
    let encoded_size = mp3_encoder
        .encode(input, mp3_out_buffer.spare_capacity_mut())
        .expect("To encode");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let encoded_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .expect("to flush");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }
    //At this point your mp3_out_buffer should have full MP3 data, ready to be written on file system or whatever
    mp3_out_buffer
}

/// test render midi to mp3
#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_render_midi() {
        let sound_font = "test_assets/Roland.sf2";
        let midi = "test_assets/abyss.mid";
        let sample_rate = 44100;
        let result = render_midi(sound_font, midi, sample_rate).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), result[1].len());
    }

    #[test]
    fn test_render_mp3() {
        let sound_font = "test_assets/Roland.sf2";
        let midi = "test_assets/abyss.mid";
        let sample_rate = 44100;
        let result = render_midi(sound_font, midi, sample_rate).unwrap();
        let id3_tag = Id3Tag {
            title: b"abyss",
            artist: b"Helium",
            album: &[],
            year: &[],
            comment: &[],
            album_art: &[],
        };
        let mp3 = render_mp3(result[0].clone(), result[1].clone(), sample_rate, id3_tag);
        assert!(!mp3.is_empty());
        let mp3_file = "rendered";
        let mut file = File::create(mp3_file).unwrap();
        file.write_all(&mp3).unwrap();
    }
}
