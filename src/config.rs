use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::ApplicationResult;

#[derive(Debug, Deserialize)]
pub struct Config {
    width: u32,
    height: u32,
    released: PathBuf,
    pressed: PathBuf,
    buttons: HashMap<ConfigButton, ButtonBounds>,
    axis: HashMap<ConfigAxis, AxisBounds>,
}

impl Config {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn released(&self) -> &Path {
        &self.released
    }

    pub fn pressed(&self) -> &Path {
        &self.pressed
    }

    pub fn buttons(&self) -> &HashMap<ConfigButton, ButtonBounds> {
        &self.buttons
    }

    pub fn axis(&self) -> &HashMap<ConfigAxis, AxisBounds> {
        &self.axis
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ConfigButton {
    A,
    B,
    X,
    Y,
    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ConfigAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    TriggerLeft,
    TriggerRight,
}

#[derive(Debug, Deserialize)]
pub struct ButtonBounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ButtonBounds {
    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Deserialize)]
pub struct AxisBounds {
    deadzone: Option<u16>,
    min: ButtonBounds,
    max: ButtonBounds,
}

impl AxisBounds {
    pub fn deadzone(&self) -> Option<u16> {
        self.deadzone
    }

    pub fn min(&self) -> &ButtonBounds {
        &self.min
    }

    pub fn max(&self) -> &ButtonBounds {
        &self.max
    }
}

pub fn load<P>(path: P) -> ApplicationResult<Config>
where
    P: AsRef<Path>,
{
    let reader = File::open(path)?;
    let config = serde_yaml::from_reader(reader)?;

    Ok(config)
}
