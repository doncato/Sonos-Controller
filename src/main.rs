use actix_files::NamedFile;
use actix_web::{error, route, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use clap::{Arg, Command};
use confy;
use env_logger::{self, Builder};
use log::LevelFilter;
use pnet::datalink::interfaces;
use serde::{Deserialize, Serialize};
use serde_json;
use sonor::{RepeatMode, Speaker};
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tokio;
use urlencoding::{decode, encode};

#[derive(Serialize, Deserialize)]
struct Config {
    path: PathBuf,
    speaker: Vec<SpeakerBox>,
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
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
            log::debug!("Connecting to {} . . .", b.ip);
            if let Some(spk) = b.to_speaker().await {
                result.push(spk);
                log::debug!("Successfully connected to {}.", b.ip);
            } else {
                log::debug!("Ignoring {}: Connection failed.", b.ip)
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
            spk.stop()
                .await
                .unwrap_or(log::debug!("Failed to stop playback for {}", self.ip));
            spk.set_volume(self.sound.volume)
                .await
                .unwrap_or(log::debug!("Failed to set volume for {}", self.ip));
            spk.set_crossfade(self.sound.crossfade)
                .await
                .unwrap_or(log::debug!("Failed to set crossfade for {}", self.ip));
            spk.set_shuffle(self.sound.shuffle)
                .await
                .unwrap_or(log::debug!("Failed to set shuffle for {}", self.ip));
            spk.set_repeat_mode(if self.sound.repeat {
                RepeatMode::All
            } else {
                RepeatMode::None
            })
            .await
            .unwrap_or(log::debug!("Failed to set repeat mode for {}", self.ip));
            spk.set_loudness(self.sound.loudness)
                .await
                .unwrap_or(log::debug!("Failed to set loudness for {}", self.ip));
            spk.set_treble(self.sound.treble)
                .await
                .unwrap_or(log::debug!("Failed to set treble for {}", self.ip));
            spk.set_bass(self.sound.bass)
                .await
                .unwrap_or(log::debug!("Failed to set bass for {}", self.ip));
            spk.clear_queue()
                .await
                .unwrap_or(log::debug!("Failed to clear playlist for {}", self.ip));

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

#[derive(Debug, Clone)]
struct OperationEnv {
    path: Box<Path>,
    spks: Vec<Speaker>,
    ips: Vec<Ipv4Addr>,
    req_ips: Vec<Ipv4Addr>,
    spks_ips: HashMap<Speaker, Ipv4Addr>,
    server_mode: bool,
}
impl OperationEnv {
    fn new(path: Box<Path>, spks: Vec<Speaker>, ips: Vec<Ipv4Addr>, server_mode: bool) -> Self {
        Self {
            path,
            spks: spks.clone(),
            ips,
            req_ips: spks
                .iter()
                .map(|spk| {
                    Ipv4Addr::from_str(spk.device().url().host().unwrap_or("0.0.0.0"))
                        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
                })
                .collect(),
            spks_ips: HashMap::new(),
            server_mode,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ApiSpeaker {
    ip: Ipv4Addr,
    trackname: String,
    trackduration: u32,
    trackelapsed: u32,
    volume: u16,
    is_playing: bool,
}
impl ApiSpeaker {
    async fn from_spk(spk: &Speaker) -> Self {
        let ip: Ipv4Addr = Ipv4Addr::from_str(spk.device().url().host().unwrap_or("0.0.0.0"))
            .unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let mut trackname = "None".to_string();
        let mut trackduration = 0;
        let mut trackelapsed = 0;
        if let Some(track) = spk.track().await.unwrap() {
            trackname = format!(
                "{} - {}",
                track.track().creator().unwrap_or("unknown"),
                track.track().title()
            );
            trackduration = track.duration();
            trackelapsed = track.elapsed();
        }
        let volume = spk.volume().await.unwrap();
        let is_playing = spk.is_playing().await.unwrap_or(false);

        Self {
            ip,
            trackname,
            trackduration,
            trackelapsed,
            volume,
            is_playing,
        }
    }
    async fn from_spks(spks: &Vec<Speaker>) -> Vec<Self> {
        let mut r: Vec<Self> = Vec::new();
        for e in spks.iter() {
            r.push(Self::from_spk(e).await)
        }
        r
    }
}

async fn init<'a>(log_level: LevelFilter, server_mode: bool) -> OperationEnv {
    Builder::new().filter(None, log_level).init();
    log::info!("Initializing . . .");

    log::debug!("Loading Config . . .");
    let cfg: Config = confy::load_path("./SonosBoxes.config").expect(
        "Failed to start because the config file could not be created or could not be read!",
    );
    let path = cfg.path.clone().into_boxed_path();
    if !path.exists() {
        panic!("Provided path in the config does not exist!")
    } else if !path.is_dir() {
        panic!("Provided path in the config is not an directory!")
    }
    log::debug!("Getting IP Addresses of the machine");
    let mut addrs: Vec<Ipv4Addr> = Vec::new();
    for iface in interfaces()
        .iter()
        .filter(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
    {
        let mut ips: Vec<Ipv4Addr> = Vec::new();
        for ip in iface.ips.iter() {
            if let ipnetwork::IpNetwork::V4(addr) = ip {
                ips.push(addr.ip())
            }
        }
        addrs.append(&mut ips)
    }
    if addrs.is_empty() {
        panic!("This machine does not have any IPv4 Address. Please make sure that all desired network-interfaces are connected to a network, have a valid IPv4 address and are accessible by this program");
    } else {
        log::info!("Found {} IP addresses", addrs.len());
        log::debug!("These IP addresses were found:\n{:#?}", addrs);
    }

    log::debug!("Trying to connect to configured speaker . . .");
    OperationEnv::new(path, cfg.to_speaker().await, addrs, server_mode)
}

#[route(
    r"/files/{filename:.*.mp3|mp4|m4a|wma|aac|ogg|flac|alac|aiff|wav}",
    method = "GET",
    method = "HEAD"
)]
async fn music_files(req: HttpRequest, state: web::Data<OperationEnv>) -> Result<NamedFile> {
    if let IpAddr::V4(src_addr) = req.peer_addr().unwrap().ip() {
        if src_addr == Ipv4Addr::LOCALHOST || state.req_ips.iter().any(|ip| ip == &src_addr) {
            let req_path = req
                .match_info()
                .query("filename")
                .parse()
                .unwrap_or("".to_string());
            let path: PathBuf =
                PathBuf::from_str(&decode(&req_path).expect("UTF-8").into_owned()).unwrap();
            let full_path = state.path.join(path);
            Ok(NamedFile::open(full_path)?)
        } else {
            Err(error::ErrorForbidden("403 - Forbidden"))
        }
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route("/api/filelist/{filename:.*}", method = "GET")]
async fn api_filelist(req: HttpRequest, state: web::Data<OperationEnv>) -> Result<impl Responder> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let mut result = Vec::new();
        let path: PathBuf = req
            .match_info()
            .query("filename")
            .parse()
            .unwrap_or(PathBuf::new());
        let full_path = state.path.join(path);
        if let Ok(elements) = full_path.read_dir() {
            let r = elements.filter(|e| e.is_ok()).collect::<Vec<_>>();
            for e in r {
                result.push({
                    let element = e.unwrap(); // It should be safe to call unwrap here, as r only contains elements which passed the 'is_ok()' check
                    let mut n = element.file_name().into_string().unwrap();
                    if element.file_type().unwrap().is_dir() {
                        n.push('/');
                    }
                    if n.starts_with('.') {
                        continue;
                    }
                    n
                });
            }
        }
        Ok(web::Json(result))
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route(r"/api/speakers/{address:.*}", method = "GET")]
async fn api_speakers(req: HttpRequest, state: web::Data<OperationEnv>) -> Result<impl Responder> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let address: Ipv4Addr = req
            .match_info()
            .query("address")
            .parse()
            .unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let speaker = ApiSpeaker::from_spks(&state.spks).await;
        if address != Ipv4Addr::BROADCAST && address != Ipv4Addr::UNSPECIFIED {
            return Ok(web::Json(
                speaker
                    .into_iter()
                    .filter(|e| e.ip == address)
                    .collect::<Vec<_>>(),
            ));
        } else {
            return Ok(web::Json(speaker));
        }
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route(
    r"/api/control/playback/{address:\d*\.\d*\.\d*\.\d*}/{action:.*}",
    method = "GET"
)]
async fn api_control_playback(
    req: HttpRequest,
    state: web::Data<OperationEnv>,
) -> Result<impl Responder> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let action: String = req.match_info().query("action").parse().unwrap();
        let address: Ipv4Addr = req.match_info().query("address").parse().unwrap();
        let spks = state
            .spks
            .clone()
            .into_iter()
            .filter(|e| {
                address
                    == Ipv4Addr::from_str(e.device().url().host().unwrap_or("0.0.0.0"))
                        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
            })
            .collect::<Vec<Speaker>>();
        let spk = spks.first().unwrap();
        match action.as_str() {
            "play" => {
                spk.play().await.unwrap();
                Ok(web::Json(()))
            }
            "pause" => {
                spk.pause().await.unwrap();
                Ok(web::Json(()))
            }
            "stop" => {
                spk.stop().await.unwrap();
                Ok(web::Json(()))
            }
            "next" => {
                spk.next().await.unwrap();
                Ok(web::Json(()))
            }
            "previous" => {
                spk.previous().await.unwrap();
                Ok(web::Json(()))
            }
            "queue-clear" => {
                spk.clear_queue().await.unwrap();
                Ok(web::Json(()))
            }
            "v-inc" => {
                spk.set_volume_relative(1).await.unwrap();
                Ok(web::Json(()))
            }
            "v-dec" => {
                spk.set_volume_relative(-1).await.unwrap();
                Ok(web::Json(()))
            }
            _ => Err(error::ErrorNotFound("404 - Not Found")),
        }
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route(
    r"/api/control/play/{address:\d*\.\d*\.\d*\.\d*}/{filename:.*}",
    method = "GET"
)]
async fn api_control_play(
    req: HttpRequest,
    state: web::Data<OperationEnv>,
) -> Result<impl Responder> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let address: Ipv4Addr = req.match_info().query("address").parse().unwrap();
        let file: PathBuf = req.match_info().query("filename").parse().unwrap();
        let speakers = state
            .spks
            .clone()
            .into_iter()
            .filter(|e| {
                address
                    == Ipv4Addr::from_str(e.device().url().host().unwrap_or("0.0.0.0"))
                        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
            })
            .collect::<Vec<Speaker>>();
        let dirty_uri = format!("{}", file.display());
        let uri = format!(
            "http://{}:46864/files/{}",
            state.ips.first().unwrap(),
            encode(&dirty_uri).into_owned()
        );

        let speaker = speakers.first().unwrap();
        log::info!("Setting uri to {}", uri);
        speaker.set_transport_uri(&uri, "").await.unwrap();
        if !speaker.is_playing().await.unwrap_or(false) {
            speaker.play().await.unwrap()
        }

        Ok(web::Json(()))
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route(
    r"/api/control/next/{address:\d*\.\d*\.\d*\.\d*}/{filename:.*}",
    method = "GET"
)]
async fn api_control_next(
    req: HttpRequest,
    state: web::Data<OperationEnv>,
) -> Result<impl Responder> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let address: Ipv4Addr = req.match_info().query("address").parse().unwrap();
        let file: PathBuf = req.match_info().query("filename").parse().unwrap();
        let speakers = state
            .spks
            .clone()
            .into_iter()
            .filter(|e| {
                address
                    == Ipv4Addr::from_str(e.device().url().host().unwrap_or("0.0.0.0"))
                        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
            })
            .collect::<Vec<Speaker>>();
        let uri = format!(
            "http://{}:46864/files/{}",
            state.ips.first().unwrap(),
            file.display()
        )
        .replace(" ", "%20");

        let speaker = speakers.first().unwrap();
        log::info!("Setting uri to {}", uri);
        speaker.queue_next(uri.as_str(), "").await.unwrap();
        if !speaker.is_playing().await.unwrap_or(false) {
            speaker.play().await.unwrap_or(());
        }

        Ok(web::Json(()))
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[route("/{filename:.*}", method = "GET")]
async fn interface(req: HttpRequest, state: web::Data<OperationEnv>) -> Result<NamedFile> {
    let src_addr = req.peer_addr().unwrap().ip();
    if (state.server_mode)
        || (src_addr.is_ipv4() && src_addr == Ipv4Addr::LOCALHOST)
        || (src_addr.is_ipv6() && src_addr == Ipv6Addr::LOCALHOST)
    {
        let mut path: PathBuf = req
            .match_info()
            .query("filename")
            .parse()
            .unwrap_or(PathBuf::new());
        if path.as_path().as_os_str().is_empty() {
            // I could use path.to_os_string() as well but then path wouldn't be borrowed
            path = Path::new("index.html").to_path_buf();
        }
        let full_path = Path::new("./web/").join(path);
        Ok(NamedFile::open(full_path)?)
    } else {
        Err(error::ErrorForbidden("403 - Forbidden"))
    }
}

#[actix_web::main]
async fn run_webhandler(op: OperationEnv) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(op.clone()))
            .service(api_filelist)
            .service(api_speakers)
            .service(api_control_playback)
            .service(api_control_play)
            .service(api_control_next)
            .service(music_files)
            .service(interface)

        //.service(actix_files::Files::new("/files/", op.path.as_ref()).show_files_listing())
    })
    .bind(("0.0.0.0", 46864))?
    .run()
    .await
}

#[tokio::main]
async fn main() {
    let args = Command::new("Sonos Controller")
        .version("0.1.0")
        .author("doncato, https://github.com/doncato")
        .about("Control one or more Sonos Speaker")
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Change log level to debug"),
        )
        .arg(
            Arg::new("server-mode")
                .short('s')
                .long("server")
                .help("Set to server mode, meaning the frontend will be reachable from anywhere"),
        )
        .get_matches();

    let llvl = if args.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let server_mode = args.is_present("server-mode");
    let op = init(llvl, server_mode).await;
    log::info!("Initialized.");
    log::debug!("Connected to {} Speaker", op.spks.len());
    log::debug!("Serving HTTP Requests to {}", op.path.display());

    let handler = thread::spawn(|| run_webhandler(op).unwrap());

    handler.join().unwrap();
}
