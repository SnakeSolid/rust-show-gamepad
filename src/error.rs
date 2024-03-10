use std::error::Error;
use std::fmt::Display;
use std::fmt::Error as FmtError;
use std::fmt::Formatter;
use std::io::Error as IoError;

use sdl2::render::TargetRenderError;
use sdl2::render::TextureValueError;
use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;
use serde_yaml::Error as YamlError;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
pub struct ApplicationError {
    message: String,
}

impl From<String> for ApplicationError {
    fn from(value: String) -> ApplicationError {
        ApplicationError {
            message: value.to_string(),
        }
    }
}

impl From<WindowBuildError> for ApplicationError {
    fn from(value: WindowBuildError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl From<IntegerOrSdlError> for ApplicationError {
    fn from(value: IntegerOrSdlError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl From<TextureValueError> for ApplicationError {
    fn from(value: TextureValueError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl From<TargetRenderError> for ApplicationError {
    fn from(value: TargetRenderError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl From<IoError> for ApplicationError {
    fn from(value: IoError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl From<YamlError> for ApplicationError {
    fn from(value: YamlError) -> ApplicationError {
        format!("{}", value).into()
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.message)
    }
}

impl Error for ApplicationError {}
