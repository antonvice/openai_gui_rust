use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use bit_vec::BitVec;
use std::fs::File;
use std::path::Path;
use std::i16;
use vad::Vad;
use audrey::write::Writer;
use std::io::Cursor;
fn main() {
    let path = Path::new("audio_in_example.wav");
    let decoder = Reader::open(path).unwrap();
    let desc = decoder.description();
    assert_eq!(1, desc.channel_count());

    // Initialize the Voice Activity Detector.
    let mut vad = Vad::new(3);

    // We need to make sure our audio is 16kHz mono.
    let frames = Linear::new([0i16], [0]);
    let signal = from_iter(decoder.samples::<i16>().map(Result::unwrap));
    let interpolator = Converter::from_hz_to_hz(signal, frames, desc.sample_rate() as f64, 16000f64);

    let window_size = 20;
    let sample_rate = 16000;

    // Window the input into chunks of 20 ms.
    for chunk in interpolator.signal().chunks_iter(window_size * sample_rate / 1000) {
        let chunk: Vec<_> = chunk.collect();
        let contains_voice = vad.contains_voice(&chunk);
        // Here we would save this chunk to a temporary file.
        // For the sake of this example, we'll print if it contains voice or not.
        println!("Contains voice: {}", contains_voice);
    }
}



fn main() {
    // Load a Wav file.
    let path = Path::new("audio_in_example.wav");
    let mut reader = Reader::open(&path).unwrap();
    let desc = reader.description();

    // Convert the audio to bytes.
    let mut data = vec![];
    for sample in reader.samples::<i16>() {
        let sample = sample.unwrap();
        data.extend_from_slice(&sample.to_ne_bytes());
    }

    // The data can now be used elsewhere (sent, stored, analyzed).
    // ...
    // For this example, we will write the data back into another Wav file.

    // Convert the bytes back to audio.
    let cursor = Cursor::new(data);
    let mut reader = Reader::new(cursor, desc.sample_rate()).unwrap();

    let path = Path::new("audio_back.wav");
    let file = File::create(&path).unwrap();
    let settings = reader.description();
    let mut writer = Writer::new(file, settings).unwrap();

    for sample in reader.samples::<i16>() {
        let sample = sample.unwrap();
        writer.write_sample(sample).unwrap();
    }
}