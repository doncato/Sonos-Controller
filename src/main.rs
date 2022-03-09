use std::net::Ipv4Addr;
use std::fmt;
use std::path::Path;
use sonor;
use confy;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct Config {
    ip: Ipv4Addr,
    sound: SoundConfig,
    
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            sound: SoundConfig::default()
        }
    }
}
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
#[derive(Serialize, Deserialize)]
struct SoundConfig {
    volume: u16,
    crossfade: bool,
    shuffle: bool,
    repeat: bool,
    loudness: bool,
    treble: i8,
    bass: i8,
}
impl ::std::default::Default for SoundConfig {
    fn default() -> Self {
        Self {
            volume: 10,
            crossfade: false,
            shuffle: false,
            repeat: false,
            loudness: false,
            treble: 5,
            bass: 5,
        }
    }
}
impl fmt::Display for SoundConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

fn init() -> Config {
    let cfg: Config = confy::load_path(Path::new("./SonosBoxes.config")).expect("Failed to start because the config file could not be created or could not be read!");
    cfg
}

fn main() {
    println!("Hello, world!");
    let cfg = init();
    println!("Config: {}", cfg);
}
