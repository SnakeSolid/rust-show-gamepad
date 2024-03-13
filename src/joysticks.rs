use std::collections::HashMap;
use std::collections::HashSet;

use sdl2::joystick::HatState;
use sdl2::joystick::Joystick;
use sdl2::JoystickSubsystem;

use crate::error::ApplicationResult;
use crate::mapping::Input;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct GuidAxis {
    giud: String,
    axis: u32,
}

impl GuidAxis {
    pub fn new(guid: &str, axis: u32) -> Self {
        GuidAxis {
            giud: guid.into(),
            axis,
        }
    }
}

#[derive(Debug)]
struct AxisLimits {
    min: i16,
    max: i16,
    trim: bool,
}

impl AxisLimits {
    pub fn new(value: i16) -> Self {
        Self {
            min: value,
            max: value,
            trim: value == i16::MIN || value == i16::MAX,
        }
    }

    pub fn extend(&mut self, value: i16) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn deadzone(&self) -> i16 {
        let range = (self.min as i32).max(self.max as i32);

        (range / 3) as i16
    }

    pub fn trim(&self) -> bool {
        self.trim
    }
}

#[derive(Debug)]
struct JoustickLimits {
    limits: HashMap<GuidAxis, AxisLimits>,
}

impl JoustickLimits {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
        }
    }

    pub fn update(&mut self, guid: &str, axis: u32, value: i16) {
        let key = GuidAxis::new(guid, axis);

        self.limits
            .entry(key)
            .or_insert_with(|| AxisLimits::new(value))
            .extend(value)
    }

    pub fn deadzone(&self, guid: &str, axis: u32) -> i16 {
        let key = GuidAxis::new(guid, axis);

        self.limits
            .get(&key)
            .map(|limits| limits.deadzone())
            .unwrap_or(0)
    }

    pub fn trim(&self, guid: &str, axis: u32) -> bool {
        let key = GuidAxis::new(guid, axis);

        self.limits
            .get(&key)
            .map(|limits| limits.trim())
            .unwrap_or(false)
    }
}

pub struct Joysticks {
    active: Option<String>,
    pressed: HashSet<Input>,
    joysticks: HashMap<u32, Joystick>,
    limits: JoustickLimits,
}

impl Joysticks {
    pub fn create(joystick_subsystem: &JoystickSubsystem) -> ApplicationResult<Self> {
        let mut joysticks = HashMap::new();

        for id in 0..joystick_subsystem.num_joysticks()? {
            let joystick = joystick_subsystem.open(id)?;

            joysticks.insert(id, joystick);
        }

        Ok(Self {
            active: None,
            pressed: HashSet::new(),
            joysticks,
            limits: JoustickLimits::new(),
        })
    }

    pub fn active(&self) -> Option<&String> {
        self.active.as_ref()
    }

    pub fn pressed(&self) -> &HashSet<Input> {
        &self.pressed
    }

    pub fn add(
        &mut self,
        joystick_subsystem: &JoystickSubsystem,
        id: u32,
    ) -> ApplicationResult<()> {
        let joystick = joystick_subsystem.open(id)?;

        self.joysticks.insert(id, joystick);

        Ok(())
    }

    pub fn remove(&mut self, id: u32) {
        self.joysticks.remove(&id);
    }

    pub fn released(&self) -> bool {
        self.pressed.is_empty()
    }

    pub fn update(&mut self) -> ApplicationResult<()> {
        self.pressed.clear();
        self.active = None;

        for joystick in self.joysticks.values() {
            let guid = joystick.guid().to_string();

            for axis in 0..joystick.num_axes() {
                let value = joystick.axis(axis)?;
                self.limits.update(&guid, axis, value);
                let deadzone = self.limits.deadzone(&guid, axis);
                let trim = self.limits.trim(&guid, axis);

                match value {
                    v if trim && v == i16::MIN || v == i16::MAX => {}
                    v if v < -deadzone => {
                        self.pressed.insert(Input::axis_min(axis));
                        self.active = Some(guid.clone());
                    }
                    v if v > deadzone => {
                        self.pressed.insert(Input::axis_max(axis));
                        self.active = Some(guid.clone());
                    }
                    _ => {}
                }
            }

            for button in 0..joystick.num_buttons() {
                let guid = joystick.guid().to_string();

                if joystick.button(button)? {
                    self.pressed.insert(Input::button(button));
                    self.active = Some(guid);
                }
            }

            for hat in 0..joystick.num_hats() {
                let guid = joystick.guid().to_string();
                let state = joystick.hat(hat)?;

                if state != HatState::Centered {
                    self.pressed.insert(Input::hat(hat, state));
                    self.active = Some(guid);
                }
            }
        }

        Ok(())
    }
}
