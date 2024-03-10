use std::collections::HashMap;

use sdl2::controller::Axis;
use sdl2::controller::Button;
use sdl2::controller::GameController;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::GameControllerSubsystem;

use crate::config::AxisBounds;
use crate::config::ButtonBounds;
use crate::config::Config;
use crate::config::ConfigAxis;
use crate::config::ConfigButton;
use crate::error::ApplicationResult;

pub struct Visualiser<'a> {
    released: Texture<'a>,
    pressed: Texture<'a>,
    mapping_button: HashMap<Button, Rect>,
    mapping_axis_min: HashMap<Axis, Rect>,
    mapping_axis_max: HashMap<Axis, Rect>,
    axis_deadzones: HashMap<Axis, u16>,
    controllers: HashMap<u32, GameController>,
}

const ALL_BUTTONS: &[Button] = &[
    Button::A,
    Button::B,
    Button::X,
    Button::Y,
    Button::Back,
    Button::Guide,
    Button::Start,
    Button::LeftStick,
    Button::RightStick,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

const ALL_AXIS: &[Axis] = &[
    Axis::LeftX,
    Axis::LeftY,
    Axis::RightX,
    Axis::RightY,
    Axis::TriggerLeft,
    Axis::TriggerRight,
];

impl<'a> Visualiser<'a> {
    pub fn create<'b, T>(
        config: &Config,
        texture_creator: &'b TextureCreator<T>,
        game_controller: &GameControllerSubsystem,
    ) -> ApplicationResult<Visualiser<'b>> {
        let released = texture_creator.load_texture(config.released())?;
        let pressed = texture_creator.load_texture(config.pressed())?;
        let mapping_button = button_maping(config.buttons());
        let (mapping_axis_min, mapping_axis_max, axis_deadzones) = axis_maping(config.axis());
        let mut controllers = HashMap::new();

        for id in 0..game_controller.num_joysticks()? {
            if game_controller.is_game_controller(id) {
                let controller = game_controller.open(id)?;

                controllers.insert(id, controller);
            }
        }

        Ok(Visualiser {
            released,
            pressed,
            mapping_button,
            mapping_axis_min,
            mapping_axis_max,
            axis_deadzones,
            controllers,
        })
    }

    pub fn controller_add(
        &mut self,
        game_controller: &GameControllerSubsystem,
        id: u32,
    ) -> ApplicationResult<()> {
        if game_controller.is_game_controller(id) {
            let controller = game_controller.open(id)?;

            self.controllers.insert(id, controller);
        }

        Ok(())
    }

    pub fn controller_remove(&mut self, id: u32) {
        self.controllers.remove(&id);
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) -> ApplicationResult<()> {
        canvas.copy(&self.released, None, None)?;

        for controller in self.controllers.values() {
            for axis in ALL_AXIS {
                let deadzone = self.axis_deadzones.get(axis).cloned().unwrap_or(8192);
                let value = controller.axis(*axis);

                match value {
                    v if (v.abs() as u16) < deadzone => {
                        if let Some(rect) = self.mapping_axis_min.get(axis).cloned() {
                            canvas.copy(&self.pressed, rect, rect)?;
                        }
                    }
                    v if (v.abs() as u16) < deadzone => {
                        if let Some(rect) = self.mapping_axis_max.get(axis).cloned() {
                            canvas.copy(&self.pressed, rect, rect)?;
                        }
                    }
                    _ => {}
                }
            }

            for button in ALL_BUTTONS {
                if controller.button(*button) {
                    if let Some(rect) = self.mapping_button.get(button).cloned() {
                        canvas.copy(&self.pressed, rect, rect)?;
                    }
                }
            }
        }

        canvas.present();

        Ok(())
    }
}

fn button_maping(buttons: &HashMap<ConfigButton, ButtonBounds>) -> HashMap<Button, Rect> {
    let mut result = HashMap::new();

    for (button, bounds) in buttons {
        let rect = bounds_to_rect(bounds);
        let button = match button {
            ConfigButton::A => Button::A,
            ConfigButton::B => Button::B,
            ConfigButton::X => Button::X,
            ConfigButton::Y => Button::Y,
            ConfigButton::Back => Button::Back,
            ConfigButton::Guide => Button::Guide,
            ConfigButton::Start => Button::Start,
            ConfigButton::LeftStick => Button::LeftStick,
            ConfigButton::RightStick => Button::RightStick,
            ConfigButton::LeftShoulder => Button::LeftShoulder,
            ConfigButton::RightShoulder => Button::RightShoulder,
            ConfigButton::DPadUp => Button::DPadUp,
            ConfigButton::DPadDown => Button::DPadDown,
            ConfigButton::DPadLeft => Button::DPadLeft,
            ConfigButton::DPadRight => Button::DPadRight,
        };

        result.insert(button, rect);
    }

    result
}

fn axis_maping(
    axis: &HashMap<ConfigAxis, AxisBounds>,
) -> (HashMap<Axis, Rect>, HashMap<Axis, Rect>, HashMap<Axis, u16>) {
    let mut axis_min = HashMap::new();
    let mut axis_max = HashMap::new();
    let mut deadzones = HashMap::new();

    for (axis, bounds) in axis {
        let min = bounds_to_rect(bounds.min());
        let max = bounds_to_rect(bounds.max());
        let axis = match axis {
            ConfigAxis::LeftX => Axis::LeftX,
            ConfigAxis::LeftY => Axis::LeftY,
            ConfigAxis::RightX => Axis::RightX,
            ConfigAxis::RightY => Axis::RightY,
            ConfigAxis::TriggerLeft => Axis::TriggerLeft,
            ConfigAxis::TriggerRight => Axis::TriggerRight,
        };

        axis_min.insert(axis, min);
        axis_max.insert(axis, max);

        if let Some(deadzone) = bounds.deadzone() {
            deadzones.insert(axis, deadzone);
        }
    }

    (axis_min, axis_max, deadzones)
}

fn bounds_to_rect(bounds: &ButtonBounds) -> Rect {
    Rect::new(
        bounds.x() as i32,
        bounds.y() as i32,
        bounds.width(),
        bounds.height(),
    )
}
