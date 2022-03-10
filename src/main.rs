use chrono::Local;
use clap::{Arg, Command};
use confy;
use env_logger::{self, Builder};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use serde_json;
use sonor::{RepeatMode, Speaker};
use std::fmt;
use std::net::Ipv4Addr;
use std::path::Path;
use tokio;

#[derive(Serialize, Deserialize)]
struct Config {
    speaker: Vec<SpeakerBox>,
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            speaker: vec![
                SpeakerBox::default(),
                SpeakerBox::default(),
                SpeakerBox::default(),
            ],
        }
    }
}
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
impl Config {
    /// Iterates over every speaker configuration in the config and converts them
    /// into a Vector of sonor::Speaker objects.
    async fn to_speaker(&self) -> Vec<Speaker> {
        let mut result = Vec::new();
        for b in self.speaker.iter() {
            if let Some(spk) = b.to_speaker().await {
                result.push(spk);
            }
        }
        result
    }
}
#[derive(Serialize, Deserialize)]
struct SpeakerBox {
    ip: Ipv4Addr,
    sound: SoundConfig,
}
impl ::std::default::Default for SpeakerBox {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            sound: SoundConfig::default(),
        }
    }
}
impl fmt::Display for SpeakerBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
impl SpeakerBox {
    async fn to_speaker(&self) -> Option<Speaker> {
        if let Some(spk) = match Speaker::from_ip(self.ip).await {
            Ok(val) => val,
            Err(err) => {
                log::error!("{:?}", err);
                None
            }
        } {
            spk.set_volume(self.sound.volume).await.unwrap_or(());
            spk.set_crossfade(self.sound.crossfade).await.unwrap_or(());
            spk.set_shuffle(self.sound.shuffle).await.unwrap_or(());
            spk.set_repeat_mode(if self.sound.repeat {
                RepeatMode::All
            } else {
                RepeatMode::None
            })
            .await
            .unwrap_or(());
            spk.set_loudness(self.sound.loudness).await.unwrap_or(());
            spk.set_treble(self.sound.treble).await.unwrap_or(());
            spk.set_bass(self.sound.bass).await.unwrap_or(());

            Some(spk)
        } else {
            log::warn!("Failed to connect to {}", self.ip);
            None
        }
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

async fn init() -> Vec<Speaker> {
    let cfg: Config = confy::load_path("./SonosBoxes.config").expect(
        "Failed to start because the config file could not be created or could not be read!",
    );
    cfg.to_speaker().await
}

#[tokio::main]
async fn main() {
    Builder::new().filter(None, LevelFilter::Info).init();

    log::info!("Hello, world!");

    let args = Command::new("Sonos Controller")
        .version("0.1.0")
        .author("doncato, https://github.com/doncato")
        .about("Control one or more Sonos Speaker")
        .arg(
            Arg::new("url")
                .required(true)
                .takes_value(true)
                .help("The url to a song to be played by the Speaker"),
        )
        .get_matches();

    let spks = init().await;
    log::info!("Connected to {} Speaker", spks.len());
    for spk in &spks {
        log::info!(
            "Loading Speaker {}",
            spk.name().await.unwrap_or("NULL".to_string())
        );
        spk.clear_queue().await.unwrap_or(());
        if let Some(url) = args.value_of("url") {
            log::info!("Adding {}", url);
            spk.set_transport_uri(url, "").await.unwrap_or(());
        } else {
            log::info!("AHHHHH");
        }
        log::info!(": {:?}", spk.track().await.unwrap());
        spk.play().await.unwrap_or(());
    }

    /*
    for spk in &spks {
        log::info!("Letting Speaker play");
        log::info!(": {:?}", spk.track().await.unwrap());
        spk.play().await.unwrap();
    }
    */
    //TODO: Add log outputs
}
