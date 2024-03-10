#![windows_subsystem = "windows"]

mod config;
mod error;
mod options;
mod visualizer;

use std::thread;
use std::time::Duration;

use error::ApplicationResult;
use options::Options;
use sdl2::event::Event;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::messagebox ;
use sdl2::messagebox:: ButtonData ;
use sdl2::messagebox:: MessageBoxButtonFlag;
use sdl2::messagebox:: MessageBoxFlag;
use sdl2::surface::Surface;
use structopt::StructOpt;
use visualizer::Visualiser;

const FRAME_TIME: Duration = Duration::from_millis(1_000 / 60);

fn main() -> ApplicationResult<()> {
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let options = Options::from_args();
    let config = config::load(options.config_path())?;
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let game_controller = sdl.game_controller()?;
    let (width, height) = Surface::from_file(config.released())?.size();
    let window = video_subsystem
        .window("Show Controller", width, height)
        .position_centered()
        .build()?;
    let mut event_pump = sdl.event_pump()?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .target_texture()
        .build()?;
    let texture_creator = canvas.texture_creator();
    let mut visualiser = Visualiser::create(&config, &texture_creator, &game_controller)?;

    let _ = messagebox::show_message_box(MessageBoxFlag::empty(), &[
        ButtonData {
            flags: MessageBoxButtonFlag::RETURNKEY_DEFAULT,
            button_id: 1,
            text: "Ok",
        }], "title", "message", None, None);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::ControllerDeviceAdded { which, .. } => {
                    visualiser.controller_add(&game_controller, which)?;
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    visualiser.controller_remove(which);
                }
                _ => {}
            }
        }

        visualiser.draw(&mut canvas)?;
        thread::sleep(FRAME_TIME);
    }

    Ok(())
}
