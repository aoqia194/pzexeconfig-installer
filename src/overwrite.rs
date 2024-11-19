use metadata::LevelFilter;
use regex::Regex;
use registry::{Hive, Security};
use std::env::current_exe;
use std::fs;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::*;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

const STEAM_REGKEY: &str = r"SOFTWARE\WOW6432Node\Valve\Steam";

const PZEXECONFIG_JSON_URL: &str = "https://gist.githubusercontent.com/aoqia194/f93a6d9cdfd66388c46ada22d067b058/raw/27deb991f4f1e785b018b73d68f4c687e5270d10/ProjectZomboid64aoqia.json";

const LIBRARY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\s*"path"\s*"(.*?)"[^}]*"apps"\s*\{[^}]*"108600"#).unwrap());

fn setup_logger() {
    let dir = current_exe().unwrap().parent().unwrap().join("logs/");

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        .filename_suffix("log")
        .build(dir)
        .unwrap();

    let file_layer = layer()
        .with_writer(file_appender)
        .compact()
        .with_ansi(false)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(false)
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_span_events(FmtSpan::FULL)
        .with_filter(LevelFilter::TRACE);

    let stdout_layer = layer()
        .with_writer(stdout)
        .with_file(false)
        .with_thread_names(false)
        .with_line_number(false)
        .with_target(true)
        .with_level(true)
        .with_span_events(FmtSpan::FULL)
        .with_filter(if cfg!(debug_assertions) {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        });

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    info!("Searching registry for Steam application install path");

    let regkey = Hive::LocalMachine
        .open(STEAM_REGKEY, Security::Read)
        .expect("Failed to open Steam app regkey");
    let install_path: PathBuf = PathBuf::from(
        regkey
            .value("InstallPath")
            .expect("Failed to read InstallPath regkey")
            .to_string(),
    );

    info!("Finding libraryfolders.vdf");

    let mut library_file = fs::File::open(install_path.join("config/libraryfolders.vdf"))
        .expect("Failed to open libraryfolders.vdf");

    let mut buf = String::new();
    library_file.read_to_string(&mut buf)?;

    if buf.is_empty() {
        panic!("Buf was empty when trying to read library file")
    }

    info!("Getting game path from libraryfolders.vdf");

    let game_path = PathBuf::from(
        LIBRARY_REGEX
            .captures(&buf)
            .expect("Failed to get regex captures")
            .get(1)
            .expect("Failed to get game path from regex")
            .as_str(),
    )
    .join("steamapps/common/ProjectZomboid");

    if !game_path.exists() {
        panic!(
            "Game path does not exist at {}!",
            game_path.to_str().unwrap()
        );
    }

    // Open file for writing at dest folder.
    info!("Overwriting ProjectZomboid64.json in game folder");

    let mut file = fs::File::create(game_path.join("ProjectZomboid64.json"))
        .expect("Failed to create config file in game folder");

    info!("Getting config from github");

    let data = reqwest::blocking::get(PZEXECONFIG_JSON_URL)
        .expect("Failed to get pzexeconfig json file from GitHub")
        .text()
        .expect("Failed to read pzexeconfig json file from GitHub");

    file.write_all(data.as_bytes())
        .expect("Failed to write all bytes to config file");

    info!("Done installing!");

    return Ok(());
}
