#![windows_subsystem = "windows"]

mod config;
mod error;
mod font;
mod mapping;
mod options;
mod visualizer;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use error::ApplicationResult;
use font::Font;
use options::Options;
use sdl2::event::Event;
use sdl2::filesystem;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::messagebox;
use sdl2::messagebox::ButtonData;
use sdl2::messagebox::MessageBoxButtonFlag;
use sdl2::messagebox::MessageBoxFlag;
use sdl2::surface::Surface;
use structopt::StructOpt;
use visualizer::Visualiser;

const FRAME_TIME: Duration = Duration::from_millis(1_000 / 60);

fn run() -> ApplicationResult<()> {
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let options = Options::from_args();
    let config = config::load(options.config_path())?;
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let joystick_subsystem = sdl.joystick()?;
    let (width, height) = Surface::from_file(config.background())?.size();
    let window = video_subsystem
        .window("Show Controller", width, height)
        .position_centered()
        .build()?;
    let mut event_pump = sdl.event_pump()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    let preferences = filesystem::pref_path("snake", "show-controller")?;
    let mut preferences = PathBuf::from(preferences);
    preferences.push("preferences.yaml");

    let font = Font::create(16, 32, &texture_creator)?;
    let mut visualiser = Visualiser::create(
        &config,
        preferences,
        &font,
        &texture_creator,
        &joystick_subsystem,
    )?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    visualiser.update_setup()?;
                }
                Event::JoyDeviceAdded { which, .. } => {
                    visualiser.joystick_add(&joystick_subsystem, which)?
                }
                Event::JoyDeviceRemoved { which, .. } => visualiser.joystick_remove(which),
                _ => {}
            }
        }

        visualiser.draw(&mut canvas)?;
        canvas.present();

        thread::sleep(FRAME_TIME);
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        let flags = MessageBoxButtonFlag::empty()
            .union(MessageBoxButtonFlag::RETURNKEY_DEFAULT)
            .union(MessageBoxButtonFlag::ESCAPEKEY_DEFAULT);
        let button = ButtonData {
            flags,
            button_id: 1,
            text: "Ok",
        };
        let _ = messagebox::show_message_box(
            MessageBoxFlag::empty(),
            &[button],
            "Error",
            &error.to_string(),
            None,
            None,
        );
    }
}
