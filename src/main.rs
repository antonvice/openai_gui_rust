use cpal::{SampleFormat, Sample};
use cpal::platform::CoreAudioStream;
use gtk::gio::BufferedInputStream;
use gtk::glib::once_cell::sync::Lazy;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Orientation, Stack, StackSwitcher};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
mod q_logic;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::thread;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.gtk_rs.GPT4CHAT";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let token_entry = Entry::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .placeholder_text("Enter your API token here...")
            .build();

        let token_button = Button::builder()
            .label("Save token and start chat")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let token_box = Box::new(Orientation::Vertical, 6);
        token_box.append(&token_entry);
        token_box.append(&token_button);

        let chat_entry = Entry::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .placeholder_text("Enter your message here...")
            .build();

        let chat_button = Button::builder()
            .label("Send message")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
            
        let record_button = Button::builder()
            .label("Record audio")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let user_message_view = gtk::TextView::builder()
            .editable(false)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .build();
        
        let assistant_message_view = gtk::TextView::builder()
            .editable(false)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .build();
        
        let chat_box = Box::new(Orientation::Vertical, 6);
        chat_box.append(&chat_entry);
        chat_box.append(&chat_button);
        chat_box.append(&user_message_view);
        chat_box.append(&assistant_message_view);
        chat_box.append(&record_button);

        let chat_entry_clone = chat_entry.clone();
        let user_message_view_clone = user_message_view.clone();
        let ass_view_clone = assistant_message_view.clone();

        let is_recording = Arc::new(Mutex::new(false));
        let is_recording_clone = is_recording.clone();
    
        // Create a buffer to store the recorded audio
        let buffer = Arc::new(Mutex::new(Vec::<i16>::new()));
        let buffer_clone = buffer.clone();
    
        // Create a channel to send the recorded audio data to the main thread
        
        record_button.connect_clicked(move |_| {
            // Toggle the recording state
            let mut recording = is_recording.lock().unwrap();
            *recording = !*recording;
    
            // If recording is started, start the input stream
            if *recording {
                start_recording(is_recording_clone.clone(), buffer_clone.clone());
            }
    
            // If recording is stopped, stop the input stream
            else {
                // Retrieve the recorded audio data from the buffer
                let audio_data: Vec<f32> = buffer.lock().unwrap().iter().map(|&sample| sample as f32 / i16::MAX as f32).collect();
    
                // Clear the buffer for the next recording
                buffer.lock().unwrap().clear();
    
                // Now you have the audio data captured from the microphone
                println!("Recorded Audio Data: {:?}", audio_data);
    
                // At this point, you can call your function to transcribe the buffer
                transcribe_audio(audio_data);
            }
        });

        chat_button.connect_clicked(move |_| {
            // Get the text from the chat_entry
            let user_message = chat_entry_clone.text().to_string();
        
            let assistant_response: String = q_logic::respond(&user_message);

            // Display the user message
            let buffer = user_message_view_clone.buffer();
            buffer.set_text(&user_message);
            let a_buffer = ass_view_clone.buffer();
            a_buffer.set_text(&assistant_response);
            // Clear the chat_entry
            chat_entry_clone.set_text("");
        });

        let stack = Stack::new();
        stack.add_titled(&token_box, Some("token"), "API Token");
        stack.add_titled(&chat_box, Some("chat"), "Chat");

        let stack_switcher = StackSwitcher::builder().stack(&stack).build();

        let vbox = Box::new(Orientation::Vertical, 6);
        vbox.append(&stack_switcher);
        vbox.append(&stack);

        let token_entry_clone = token_entry.clone();
        let stack_clone = stack.clone();
        token_button.connect_clicked(move |_| {
            let token = token_entry_clone.text().to_string();
            env::set_var("OPENAI_API_KEY", &token);
            stack_clone.set_visible_child_name("chat");
        });

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Chat App")
            .child(&vbox)
            .build();

        window.present();
    });

    app.run();
}

fn start_recording(is_recording: Arc<Mutex<bool>>, buffer: Arc<Mutex<Vec<i16>>>) {
    // Get the default audio host
    let host = cpal::default_host();

    // Get the default input device (microphone)
    let input_device = host
        .default_input_device()
        .expect("No input device found.");

    // Print the name of the input device
    println!("Input Device: {}", input_device.name().unwrap());

    // Get the supported input configurations
    let mut supported_configs = match input_device.supported_input_configs() {
        Ok(configs) => configs,
        Err(err) => {
            eprintln!("Error getting supported input configurations: {}", err);
            return;
        }
    };

    // Check if there are supported configurations
    if let Some(supported_config) = supported_configs.next() {
        // Print the supported configurations
        println!("Supported Configurations:");
        for (index, config) in supported_configs.enumerate() {
            println!("Configuration {}: {:?}", index + 1, config);
        }
    } else {
        eprintln!("No supported configurations found for the input device.");
        return;
    }

    let mut supported_configs_range = input_device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    // Manually select a supported configuration based on your requirements
    let config = supported_config.into();
    // Create an input stream with the specified format and callback function
    let input_stream = input_device
        .build_input_stream(&config, move |data, info| {
            input_callback(data, info, &is_recording, &buffer);
        }, |err| eprintln!("Error in input stream: {:?}", err), None)
        .expect("Failed to build input stream.");

    // Start the input stream
    input_stream
        .play()
        .expect("Failed to start input stream.");
}

fn input_callback(data: &[i16], _: &cpal::InputCallbackInfo, is_recording: &Arc<Mutex<bool>>, buffer: &Arc<Mutex<Vec<f32>>>) {
    // Only capture audio when recording is enabled
    if *is_recording.lock().unwrap() {
        // Convert i16 data to f32 and append audio data to the buffer
        let converted_data: Vec<f32> = data.iter().map(|&sample| sample as f32 / i16::MAX as f32).collect();
        buffer.lock().unwrap().extend_from_slice(&converted_data);
    }
}

fn transcribe_audio(audio_data: Vec<f32>) {
    // Load the Whisper context and model
    let ctx = WhisperContext::new("model/ggml-base.en.bin")
        .expect("failed to load model");
    let mut state = ctx.create_state().expect("failed to create key");

    // Create parameters for the model
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_n_threads(1);
    params.set_translate(true);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Convert audio data to floating point samples
    let audio = whisper_rs::convert_integer_to_float_audio(&audio_data);

    // Convert audio to 16KHz mono f32 samples, as required by the model
    let audio = whisper_rs::convert_stereo_to_mono_audio(&audio).unwrap();

    // Run the Whisper model
    state.full(params, &audio[..]).expect("failed to run model");

    // Create a file to write the transcript to
    let mut file = File::create("transcript.txt").expect("failed to create file");

    // Iterate through the segments of the transcript
    let num_segments = state.full_n_segments().expect("failed to get number of segments");
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i).expect("failed to get segment");
        let start_timestamp = state.full_get_segment_t0(i).expect("failed to get start timestamp");
        let end_timestamp = state.full_get_segment_t1(i).expect("failed to get end timestamp");

        // Print the segment to stdout
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);

        // Format the segment information as a string
        let line = format!("[{} - {}]: {}\n", start_timestamp, end_timestamp, segment);

        // Write the segment information to the file
        file.write_all(line.as_bytes()).expect("failed to write to file");
    }
}