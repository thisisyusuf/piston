//! soundstream.rs
//!
//! A real-time soundstream demo that smoothly copies input
//! from a microphone straight to the output. If <Space> is
//! pressed, the SoundStream thread will begin to calculate
//! and print the samplerate to demonstrate the inter-task
//! event handling.
//!
//! Note: Beware of feedback!

#![feature(globs)]

extern crate piston;

use piston::{
    keyboard,
    AssetStore,
    Game,
    GameWindow,
    GameWindowSDL2,
    GameWindowSettings,
    KeyPressArgs,
    SoundStream,
    SoundStreamSettings
};

// Structs
//------------------------------

/// Main application struct.
pub struct App {
    /// Channel for sending information to the audio stream.
    kill_chan: Option<Sender<bool>> // Channel for sending kill message.
}

/// The audio is non-blocking and needs it's own struct.
pub struct AppSoundStream {
    /// Channel for receiving game events from main game stream.
    kill_chan: Option<Receiver<bool>>, // Channel for receiving kill message.
    should_exit: bool, // Trigger for closing the stream.
    should_print: bool, // Toggle for printing the sample_rate.
    buffer: Vec<f32> // Buffer for passing input to output.
}

// Game Method Implementations
//------------------------------

impl Game for App {

    /// Setup / load the app stuff ready for the main loop.
    /// If using a SoundStream, it must be created within this method.
    fn load(&mut self, asset_store: &mut AssetStore) {

        // Create a channel for communicating events with the soundstream.
        // Note: this channel is used for sending InteractiveEvents, but
        // the same technique could be used here to create custom channels
        // that can safely send any kind of unique data.
        let (send, recv) = channel();
        self.kill_chan = Some(send);

        // Create the soundstream on it's own thread for non-blocking, real-time audio.
        // "soundstreamer" will setup and iterate soundstream using portaudio.
        spawn(proc() {
            let mut soundstream =
                AppSoundStream::new(Some(recv)).run(SoundStreamSettings::cd_quality());
        });

    }

    /// Keypress callback.
    fn key_press(&mut self, args: &KeyPressArgs) {
        println!("Game thread key: {}", args.key);
    }

    /*
    /// Specify the event sending channel. This must be done if we wish
    /// to send interactive events to the SoundStream.
    fn get_event_sender(&self) -> Option<Sender<GameEvent<'static>>> {
        self.stream_chan.clone()
    }
    */
}

impl Drop for App {
    /// Tell the soundstream to exit when App is destroyed.
    fn drop(&mut self) {
        let chan = self.kill_chan.clone();
        match chan {
            Some(sender) => sender.send(true),
            None => ()
        }
    }
}

impl App {
    /// Creates a new application.
    pub fn new() -> App {
        App {
            kill_chan: None
        }
    }
}

// SoundStream Method Implementations
//------------------------------

impl SoundStream for AppSoundStream {

    /// Load (called prior to main soundstream loop).
    fn load(&mut self) {
        println!("Press <Spacebar> to start/stop printing the real-time sample rate.");
    }

    /// Update (gets called prior to audio_in/audio_out).
    fn update(&mut self, settings: &SoundStreamSettings, dt: u64) {
        if self.should_print {
            let dtsec: f64 = dt as f64 / 1000000000f64;
            println!("Real-time sample rate: {}", (1f64 / dtsec) * settings.frames as f64);
        }
        match self.kill_chan {
            Some(ref receiver) => match receiver.try_recv() {
                Ok(_) => self.should_exit = true,
                Err(_) => ()
            },
            None => ()
        }
    }

    /// AudioInput
    fn audio_in(&mut self, input: &Vec<f32>, settings: &SoundStreamSettings) {
        self.buffer = input.clone();
    }

    /// AudioOutput
    fn audio_out(&mut self, output: &mut Vec<f32>, settings: &SoundStreamSettings) {
        *output = self.buffer.clone()
    }

    /// KeyPress
    fn key_press(&mut self, args: &KeyPressArgs) {
        println!("Soundstream thread key: {}", args.key);
        if args.key == keyboard::Space {
            let print = if self.should_print { false } else { true };
            self.should_print = print;
        }
        if args.key == keyboard::Escape {
            self.should_exit = true;
        }
    }

    /*
    /// Retrieve Events for callback (i.e. mouse, keyboard).
    fn check_for_event(&self) -> Option<GameEvent<'static>> {
        match self.chan {
            Some(ref receiver) => match receiver.try_recv() {
                Ok(event) => Some(event),
                Err(_) => None
            },
            None => None
        }
    }
    */

    /// Setup the exit condition (is checked once per buffer).
    fn exit(&self) -> bool { self.should_exit }

}

impl AppSoundStream {
    /// AppSoundStream constructor.
    pub fn new(recv: Option<Receiver<bool>>) -> AppSoundStream {
        AppSoundStream {
            kill_chan: recv,
            should_exit: false,
            should_print: false,
            buffer: vec![]
        }
    }
}

// Main
//------------------------------

#[start]
fn start(argc: int, argv: **u8) -> int {
    // Run gui on the main thread.
    native::start(argc, argv, main)
}

fn main() {
    let mut window: GameWindowSDL2 = GameWindow::new(
        GameWindowSettings {
            title: "soundstream".to_string(),
            size: [300, 300],
            fullscreen: false,
            exit_on_esc: true,
            background_color: [0.1, 0.1, 0.1, 0.1],
        }
    );

    let mut asset_store = AssetStore::from_folder("assets");
    let mut app = App::new();
    app.run(&mut window, &mut asset_store);
}


//------------------------------
