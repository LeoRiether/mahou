/// Mostly copied from https://github.com/DeGuitard/anime-cli/
/// Error handling is kind of whack...
pub mod irc;

use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpStream};
use std::path::Path;
use std::str::from_utf8;
use std::{fmt, thread};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Connection error: {0}")]
    Connection(io::Error),

    #[error("Couldn't create file '{0}' due to: {1}")]
    FileCreation(String, io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub fn download(
    entry: &crate::finder::Entry,
    config: irc::Config,
    directory: impl AsRef<Path>,
) -> Result<()> {
    connect_and_download(irc::Request {
        config,
        bot: entry.bot_name.clone(),
        packages: vec![entry.package_number.to_string()],
        directory: directory.as_ref(),
    })
}

fn connect_and_download(request: irc::Request) -> Result<()> {
    let multibar = MultiProgress::new();
    let new_progressbar = |total_bytes: u64| {
        let pb = ProgressBar::new(total_bytes);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
        pb
    };

    let mut download_handles = Vec::new();
    let mut has_joined = false;
    let mut stream = log_in(&request)?;

    let mut message_buffer = String::new();
    while download_handles.len() < request.packages.len() {
        let message = read_next_message(&mut stream, &mut message_buffer)?;

        if irc::PING_REGEX.is_match(&message) {
            let pong = message.replace("PING", "PONG");
            stream.write_all(pong.as_bytes())?;
            if !has_joined {
                let channel_join_cmd = format!("JOIN #{}\r\n", request.config.channel);
                stream.write_all(channel_join_cmd.as_bytes())?;
                has_joined = true;
            }
        }
        if irc::JOIN_REGEX.is_match(&message) {
            for package in &request.packages {
                let xdcc_send_cmd = format!("PRIVMSG {} :xdcc send #{}\r\n", request.bot, package);
                stream.write_all(xdcc_send_cmd.as_bytes())?;
            }
        }
        if irc::DCC_SEND_REGEX.is_match(&message) {
            let directory = request.directory.to_owned();
            let request = parse_dcc_send(&message);
            let bar = multibar.add(new_progressbar(request.file_size as u64));
            let handle = thread::spawn(move || download_file(request, bar, directory));
            download_handles.push(handle);
        }
    }
    stream.write_all("QUIT :my job is done here!\r\n".as_bytes())?;
    stream.shutdown(Shutdown::Both).unwrap();
    download_handles.into_iter().try_for_each(|handle| {
        handle
            .join()
            .map_err(|e| e.downcast::<Error>().unwrap())
            .unwrap()
    })?;
    Ok(())
}

fn log_in(request: &irc::Request) -> Result<TcpStream> {
    let mut stream = TcpStream::connect(&request.config.server).map_err(Error::Connection)?;
    stream.write_all(format!("NICK {}\r\n", request.config.nickname).as_bytes())?;
    stream.write_all(
        format!(
            "USER {} 0 * {}\r\n",
            request.config.nickname, request.config.nickname
        )
        .as_bytes(),
    )?;
    Ok(stream)
}

fn read_next_message(stream: &mut TcpStream, message_builder: &mut String) -> Result<String> {
    let mut buffer = [0; 4];
    while !message_builder.contains('\n') {
        let count = stream.read(&mut buffer[..])?;
        message_builder.push_str(from_utf8(&buffer[..count]).unwrap_or_default());
    }
    let endline_offset = message_builder.find('\n').unwrap() + 1;
    let message = message_builder.get(..endline_offset).unwrap().to_string();
    message_builder.replace_range(..endline_offset, "");
    Ok(message)
}

fn parse_dcc_send(message: &str) -> irc::DCCSend {
    let captures = irc::DCC_SEND_REGEX.captures(message).unwrap();
    let ip_number = captures[2].parse::<u32>().unwrap();
    irc::DCCSend {
        filename: captures[1].to_string(),
        ip: IpAddr::V4(Ipv4Addr::from(ip_number)),
        port: captures[3].to_string(),
        file_size: captures[4].parse::<usize>().unwrap(),
    }
}

fn download_file(
    request: irc::DCCSend,
    bar: ProgressBar,
    directory: impl AsRef<Path>,
) -> Result<()> {
    let path = directory.as_ref().join(&request.filename);
    let mut file = File::create(&path)
        .map_err(|e| Error::FileCreation(path.to_string_lossy().to_string(), e))?;
    let mut stream = TcpStream::connect(format!("{}:{}", request.ip, request.port))
        .map_err(Error::Connection)?;
    let mut buffer = [0; 8192];

    let mut bytes: usize = 0;

    while bytes < request.file_size {
        let count = stream.read(&mut buffer[..])?;
        file.write_all(&buffer[..count])?;
        bytes += count;
        bar.set_position(bytes as u64);
    }
    bar.finish_with_message(format!("Done downloading {}", request.filename));
    stream.shutdown(Shutdown::Both)?;
    file.flush()?;
    Ok(())
}
