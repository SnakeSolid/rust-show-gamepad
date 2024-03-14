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
pub struct AxisLimits {
    default: i32,
    min: i32,
    max: i32,
}

impl AxisLimits {
    pub fn new(value: i16) -> Self {
        Self {
            default: value as i32,
            min: value as i32,
            max: value as i32,
        }
    }

    pub fn extend(&mut self, value: i16) {
        self.min = self.min.min(value as i32);
        self.max = self.max.max(value as i32);
    }

    pub fn zone(&self, value: i16) -> AxisZone {
        let bound = (self.min).max(self.max) / 4;

        match value as i32 {
            v if (v - self.default).abs() < bound => AxisZone::Default,
            v if v < self.default => AxisZone::Min,
            v if v > self.default => AxisZone::Max,
            _ => AxisZone::Default,
        }
    }
}

#[derive(Debug)]
pub enum AxisZone {
    Min,
    Default,
    Max,
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

    pub fn reset(&mut self) {
        self.limits.clear();
    }

    pub fn update(&mut self, guid: &str, axis: u32, value: i16) {
        let key = GuidAxis::new(guid, axis);

        self.limits
            .entry(key)
            .or_insert_with(|| AxisLimits::new(value))
            .extend(value)
    }

    pub fn zone(&self, guid: &str, axis: u32, value: i16) -> AxisZone {
        let key = GuidAxis::new(guid, axis);

        self.limits
            .get(&key)
            .map(|limits| limits.zone(value))
            .unwrap_or(AxisZone::Default)
    }
}

pub struct Joysticks {
    active: Option<String>,
    pressed: HashSet<Input>,
    keyboard: HashSet<Input>,
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
            keyboard: HashSet::new(),
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

    pub fn reset_limits(&mut self) {
        self.limits.reset();
    }

    pub fn key_down(&mut self, key: &str) {
        self.keyboard.insert(Input::key(key));
    }

    pub fn key_up(&mut self, key: &str) {
        self.keyboard.remove(&Input::key(key));
    }

    pub fn update(&mut self) -> ApplicationResult<()> {
        self.pressed.clear();
        self.active = None;

        for joystick in self.joysticks.values() {
            let guid = joystick.guid().to_string();

            for axis in 0..joystick.num_axes() {
                let value = joystick.axis(axis)?;
                self.limits.update(&guid, axis, value);
                let zone = self.limits.zone(&guid, axis, value);

                match zone {
                    AxisZone::Min => {
                        self.pressed.insert(Input::axis_min(axis));
                        self.active = Some(guid.clone());
                    }
                    AxisZone::Max => {
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

        if !self.keyboard.is_empty() {
            self.pressed.extend(self.keyboard.iter().cloned());
            self.active = Some("Keyboard".into());
        }

        Ok(())
    }
}
