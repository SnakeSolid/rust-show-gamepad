use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "show-gamepad")]
pub struct Options {
    #[structopt(
        short = "c",
        long = "config-path",
        name = "CONFIG_PATH",
        help = "Use CONFIG_PATH as configuration file",
        default_value = "config.yaml",
        parse(from_os_str)
    )]
    config_path: PathBuf,
}

impl Options {
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}
