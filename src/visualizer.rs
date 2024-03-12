use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use sdl2::image::LoadTexture;
use sdl2::joystick::Joystick;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::JoystickSubsystem;

use crate::config::Config;
use crate::error::ApplicationResult;
use crate::font::Font;
use crate::mapping::Input;
use crate::mapping::Mapping;

pub struct Visualiser<'a> {
    background: Texture<'a>,
    sprites: HashMap<usize, Sprite<'a>>,
    preferences: PathBuf,
    font: &'a Font<'a>,
    show_help: bool,
    mapping: Mapping,
    joysticks: HashMap<u32, Joystick>,
    setup: SetupOverlay,
}

impl<'a> Visualiser<'a> {
    pub fn create<'b, T>(
        config: &Config,
        preferences: PathBuf,
        font: &'b Font,
        texture_creator: &'b TextureCreator<T>,
        joystick_subsystem: &JoystickSubsystem,
    ) -> ApplicationResult<Visualiser<'b>> {
        let background = texture_creator.load_texture(config.background())?;
        let mut sprites = HashMap::new();

        for (id, sprite) in config.sprites().iter().enumerate() {
            let group = sprite.group();
            let name = sprite.name();
            let sprite = texture_creator.load_texture(sprite.path())?;

            sprites.insert(id.clone(), Sprite::new(group, name, sprite));
        }

        let n_sprites = sprites.len();
        let mut joysticks = HashMap::new();

        for id in 0..joystick_subsystem.num_joysticks()? {
            let joystick = joystick_subsystem.open(id)?;

            joysticks.insert(id, joystick);
        }

        let mapping = match preferences.exists() {
            true => Mapping::load(&preferences)?,
            false => Mapping::new(),
        };

        Ok(Visualiser {
            background,
            sprites,
            preferences,
            font,
            show_help: true,
            mapping,
            joysticks,
            setup: SetupOverlay::new(n_sprites),
        })
    }

    pub fn joystick_add(
        &mut self,
        joystick_subsystem: &JoystickSubsystem,
        id: u32,
    ) -> ApplicationResult<()> {
        let joystick = joystick_subsystem.open(id)?;

        self.joysticks.insert(id, joystick);

        Ok(())
    }

    pub fn joystick_remove(&mut self, id: u32) {
        self.joysticks.remove(&id);
    }

    pub fn update_setup(&mut self) -> ApplicationResult<()> {
        if self.setup.enabled() {
            if let Some(input_state) = self.setup.pressed() {
                let guid = input_state.guid().into();
                let buttons = input_state.pressed();
                let sprite = self.setup.current_sprite;

                self.mapping.push(guid, buttons, sprite);
            }

            if !self.setup.next_sprite() {
                self.mapping.save(&self.preferences)?;
            }
        } else {
            self.setup.enable();
        }

        Ok(())
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> ApplicationResult<()> {
        let mut pressed = HashSet::new();
        let mut active_guid = None;

        for joystick in self.joysticks.values() {
            let guid = joystick.guid().to_string();

            for axis in 0..joystick.num_axes() {
                let value = joystick.axis(axis)?;

                match value {
                    v if v < -8192 => {
                        pressed.insert(Input::axis_min(axis));
                        active_guid = Some(guid.clone());
                    }
                    v if v > 8192 => {
                        pressed.insert(Input::axis_max(axis));
                        active_guid = Some(guid.clone());
                    }
                    _ => {}
                }
            }

            for button in 0..joystick.num_buttons() {
                let guid = joystick.guid().to_string();

                if joystick.button(button)? {
                    pressed.insert(Input::button(button));
                    active_guid = Some(guid);
                }
            }
        }

        canvas.copy(&self.background, None, None)?;

        if self.show_help {
                self.font.write(
                    canvas,
                    8,
                    8,
                    "Use `Return` key to start button mapping.\nPress any button to hide message.",
                )?;
        }

        if self.setup.enabled() {
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 192));
            canvas.fill_rect(None)?;

            let sprite = self.setup.current_sprite();

            if let Some(sprite) = self.sprites.get(&sprite) {
                canvas.copy(&sprite.texture(), None, None)?;

                self.font.write(
                    canvas,
                    8,
                    8,
                    &format!("Binding input for {}", sprite.name()),
                )?;
            }

            if let Some(guid) = active_guid {
                let mut buttons: Vec<_> = pressed.iter().map(ToString::to_string).collect();
                buttons.sort();

                self.setup.set_pressed(guid, pressed);
                self.font.write(
                    canvas,
                    8,
                    48,
                    &format!("Active keys: {}", buttons.join(", ")),
                )?;
            } else {
                self.font.write(canvas, 8, 48, &format!("No active keys"))?;
            }
        } else {
            if let Some(giud) = active_guid {
                let mut groups = HashSet::new();
                let sprites = self.mapping.sprites(&giud, &pressed);

                for sprite in sprites {
                    if let Some(sprite) = self.sprites.get(&sprite) {
                        if groups.insert(sprite.group()) {
                            canvas.copy(&sprite.texture(), None, None)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InputState {
    guid: String,
    pressed: HashSet<Input>,
}

impl InputState {
    pub fn new(guid: String, pressed: HashSet<Input>) -> Self {
        Self { guid, pressed }
    }

    pub fn guid(&self) -> &str {
        &self.guid
    }

    pub fn pressed(&self) -> &HashSet<Input> {
        &self.pressed
    }
}

#[derive(Debug)]
struct SetupOverlay {
    n_sprites: usize,
    enabled: bool,
    current_sprite: usize,
    pressed: Option<InputState>,
}

impl SetupOverlay {
    pub fn new(n_sprites: usize) -> Self {
        Self {
            n_sprites,
            enabled: false,
            current_sprite: 0,
            pressed: None,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn current_sprite(&self) -> usize {
        self.current_sprite
    }

    pub fn pressed(&mut self) -> Option<InputState> {
        self.pressed.take()
    }

    pub fn set_pressed(&mut self, guid: String, pressed: HashSet<Input>) {
        self.pressed = Some(InputState::new(guid, pressed));
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.current_sprite = 0;
        self.pressed = None;
    }

    pub fn next_sprite(&mut self) -> bool {
        self.current_sprite += 1;
        self.enabled = self.current_sprite < self.n_sprites;
        self.enabled
    }
}

struct Sprite<'a> {
    group: usize,
    name: String,
    texture: Texture<'a>,
}

impl<'a> Sprite<'a> {
    pub fn new<'b>(group: usize, name: &str, texture: Texture<'b>) -> Sprite<'b> {
        Sprite {
            group,
            name: name.into(),
            texture,
        }
    }

    pub fn group(&self) -> usize {
        self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
