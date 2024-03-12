use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::error::ApplicationResult;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
enum Direction {
    Minimum,
    Maximum,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Input {
    Button { button: u32 },
    Axis { axis: u32, direction: Direction },
}

impl Input {
    pub fn button(button: u32) -> Self {
        Input::Button { button }
    }

    pub fn axis_min(axis: u32) -> Self {
        Input::Axis {
            axis,
            direction: Direction::Minimum,
        }
    }

    pub fn axis_max(axis: u32) -> Self {
        Input::Axis {
            axis,
            direction: Direction::Maximum,
        }
    }
}

impl ToString for Input {
    fn to_string(&self) -> String {
        match self {
            Input::Button { button } => format!("b{}", button),
            Input::Axis {
                axis,
                direction: Direction::Minimum,
            } => format!("a{} min", axis),
            Input::Axis {
                axis,
                direction: Direction::Maximum,
            } => format!("a{} max", axis),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    joysticks: HashMap<String, Vec<SpriteMapping>>,
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            joysticks: HashMap::new(),
        }
    }

    pub fn push(&mut self, guid: &str, pressed: &HashSet<Input>, sprite: usize) {
        let entry = self.joysticks.entry(guid.into()).or_insert_with(Vec::new);

        if pressed.is_empty() {
            entry.retain(|sm| sm.sprite() == sprite);
        } else {
            let sprite_mapping = SpriteMapping::new(pressed.clone(), sprite);

            entry.push(sprite_mapping);
            entry.sort_by_key(|sm| (-(sm.buttons.len() as isize), sm.sprite()));
        }
    }

    pub fn sprites(&self, giud: &str, pressed: &HashSet<Input>) -> Vec<usize> {
        let mut result = Vec::new();

        if let Some(list) = self.joysticks.get(giud) {
            for sprite_mapping in list {
                if sprite_mapping.buttons().is_superset(pressed) {
                    result.push(sprite_mapping.sprite());
                }
            }
        }

        result
    }

    pub fn load<P>(path: P) -> ApplicationResult<Self>
    where
        P: AsRef<Path>,
    {
        let reader = File::open(path)?;
        let mapping = serde_yaml::from_reader(reader)?;

        Ok(mapping)
    }

    pub fn save<P>(&self, path: P) -> ApplicationResult<()>
    where
        P: AsRef<Path>,
    {
        let writer = File::create(path)?;
        serde_yaml::to_writer(writer, self)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SpriteMapping {
    buttons: HashSet<Input>,
    sprite: usize,
}

impl SpriteMapping {
    pub fn new(buttons: HashSet<Input>, sprite: usize) -> Self {
        Self { buttons, sprite }
    }

    pub fn buttons(&self) -> &HashSet<Input> {
        &self.buttons
    }

    pub fn sprite(&self) -> usize {
        self.sprite
    }
}
