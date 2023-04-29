use lazy_static::lazy_static;
use regex::Regex;
use std::{net::IpAddr, path::Path};

lazy_static! {
    pub static ref DCC_SEND_REGEX: Regex =
        Regex::new(r#"DCC SEND "?(.*)"? (?:(\d+)|((?:[0-9a-fA-F]*:){2,}[0-9a-fA-F]*)) (\d+) (\d+)"#).unwrap();
    pub static ref PING_REGEX: Regex = Regex::new(r#"PING :\d+"#).unwrap();
    pub static ref JOIN_REGEX: Regex = Regex::new(r#"JOIN :#.*"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: String,
    pub channel: String,
    pub nickname: String,
}

pub struct Request<'p> {
    pub config: Config,
    pub bot: String,
    pub packages: Vec<String>,
    pub directory: &'p Path,
}

pub struct DCCSend {
    pub filename: String,
    pub ip: IpAddr,
    pub port: String,
    pub file_size: usize,
}
