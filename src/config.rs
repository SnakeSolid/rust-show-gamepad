use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::ApplicationResult;

#[derive(Debug, Deserialize)]
pub struct Config {
    background: PathBuf,
    sprites: Vec<Sprite>,
}

impl Config {
    pub fn background(&self) -> &Path {
        &self.background
    }

    pub fn sprites(&self) -> &[Sprite] {
        &self.sprites
    }
}

#[derive(Debug, Deserialize)]
pub struct Sprite {
    group: usize,
    name: String,
    path: PathBuf,
    #[serde(default)]
    default: bool,
}

impl Sprite {
    pub fn group(&self) -> usize {
        self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn default(&self) -> bool {
        self.default
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
