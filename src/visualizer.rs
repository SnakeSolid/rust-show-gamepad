use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use sdl2::image::LoadTexture;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::JoystickSubsystem;

use crate::config::Config;
use crate::error::ApplicationResult;
use crate::font::Font;
use crate::joysticks::Joysticks;
use crate::mapping::Mapping;

pub struct Visualiser<'a> {
    background: Texture<'a>,
    sprites: HashMap<usize, Sprite<'a>>,
    default: HashSet<usize>,
    preferences: PathBuf,
    font: &'a Font<'a>,
    show_help: bool,
    mapping: Mapping,
    joysticks: Joysticks,
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
        let mut default = HashSet::new();

        for (id, sprite) in config.sprites().iter().enumerate() {
            let group = sprite.group();
            let name = sprite.name();
            let texture = texture_creator.load_texture(sprite.path())?;

            sprites.insert(id.clone(), Sprite::new(group, name, texture));

            if sprite.default() {
                default.insert(id.clone());
            }
        }

        let n_sprites = sprites.len();
        let mapping = match preferences.exists() {
            true => Mapping::load(&preferences)?,
            false => Mapping::new(),
        };

        Ok(Visualiser {
            background,
            sprites,
            default,
            preferences,
            font,
            show_help: true,
            mapping,
            joysticks: Joysticks::create(joystick_subsystem)?,
            setup: SetupOverlay::new(n_sprites),
        })
    }

    pub fn joystick_add(
        &mut self,
        joystick_subsystem: &JoystickSubsystem,
        id: u32,
    ) -> ApplicationResult<()> {
        self.joysticks.add(joystick_subsystem, id)
    }

    pub fn joystick_remove(&mut self, id: u32) {
        self.joysticks.remove(id);
    }

    pub fn update(&mut self) -> ApplicationResult<()> {
        self.joysticks.update()
    }

    pub fn update_setup(&mut self) -> ApplicationResult<()> {
        if self.setup.enabled() {
            if let Some(guid) = self.joysticks.active() {
                let pressed = self.joysticks.pressed();
                let sprite = self.setup.current_sprite();

                self.mapping.push(guid, pressed, sprite);
            }

            if !self.setup.next_sprite() {
                self.mapping.save(&self.preferences)?;
                self.show_help = false;
            }
        } else {
            self.setup.enable();
        }

        Ok(())
    }

    pub fn reset_limits(&mut self) {
        self.joysticks.reset_limits();
    }

    pub fn key_down(&mut self, scancode: Scancode) {
        self.joysticks.key_down(scancode as u32);
        self.show_help = false;
    }

    pub fn key_up(&mut self, scancode: Scancode) {
        self.joysticks.key_up(scancode as u32);
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> ApplicationResult<()> {
        canvas.copy(&self.background, None, None)?;

        if self.show_help {
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 192));
            canvas.fill_rect(None)?;
            self.font.write(
                canvas,
                8,
                8,
                "Use F1 to start mapping.\nPress any button to hide message.",
            )?;

            self.show_help = !self.joysticks.released() || !self.setup.enabled();
        }

        if self.setup.enabled() {
            let pressed = self.joysticks.pressed();
            let sprite = self.setup.current_sprite();

            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 192));
            canvas.fill_rect(None)?;

            if let Some(sprite) = self.sprites.get(&sprite) {
                canvas.copy(&sprite.texture(), None, None)?;

                self.font.write(
                    canvas,
                    8,
                    8,
                    &format!("Binding input for {}.", sprite.name()),
                )?;
            }

            if !pressed.is_empty() {
                let mut buttons: Vec<_> = pressed.iter().map(ToString::to_string).collect();
                buttons.sort();

                self.font.write(
                    canvas,
                    8,
                    48,
                    &format!("Active keys: {}", buttons.join(", ")),
                )?;
                self.font
                    .write(canvas, 8, 88, "Press F1 to save, F2 reset limit.")?;
            } else {
                self.font.write(canvas, 8, 48, &format!("No active keys"))?;
                self.font
                    .write(canvas, 8, 88, "Press F1 to skip, F2 reset limit.")?;
            }
        } else {
            if let Some(giud) = self.joysticks.active() {
                let mut groups = HashSet::new();
                let pressed = self.joysticks.pressed();
                let sprites = self.mapping.sprites(&giud, pressed);

                for sprite in sprites {
                    if let Some(sprite) = self.sprites.get(&sprite) {
                        if groups.insert(sprite.group()) {
                            canvas.copy(&sprite.texture(), None, None)?;
                        }
                    }
                }

                for sprite in self.default.iter().flat_map(|i| self.sprites.get(i)) {
                    if groups.insert(sprite.group()) {
                        canvas.copy(&sprite.texture(), None, None)?;
                    }
                }

                let mut buttons: Vec<_> = pressed.iter().map(ToString::to_string).collect();
                buttons.sort();

                self.font.write(canvas, 8, 256, &buttons.join(", "))?;
                self.font.write(canvas, 8, 288, &format!("{:?}", groups))?;
            } else if !self.show_help {
                let mut groups = HashSet::new();

                for sprite in self.default.iter().flat_map(|i| self.sprites.get(i)) {
                    if groups.insert(sprite.group()) {
                        canvas.copy(&sprite.texture(), None, None)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct SetupOverlay {
    n_sprites: usize,
    enabled: bool,
    current_sprite: usize,
}

impl SetupOverlay {
    pub fn new(n_sprites: usize) -> Self {
        Self {
            n_sprites,
            enabled: false,
            current_sprite: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn current_sprite(&self) -> usize {
        self.current_sprite
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.current_sprite = 0;
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
