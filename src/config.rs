use std::path::{Path, PathBuf};
use iron;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use toml;

pub struct BasePath;

impl iron::typemap::Key for BasePath {
    type Value = PathBuf;
}

#[derive(Deserialize)]
pub struct Configuration {
    pub base_path: PathBuf,
    pub user_path: PathBuf,
    pub access_path: PathBuf,
    pub serve: String
}

impl Configuration {
    pub fn load(path: PathBuf) -> Result<Configuration, Box<Error>> {
        let mut data = String::new();
        File::open(path)?.read_to_string(&mut data);
        Ok(toml::from_str(&data)?)
    }
}
