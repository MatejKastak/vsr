use chrono::{Local, SecondsFormat};
use clap::Parser;
use color_eyre::eyre::{ContextCompat, Result};
use dirs::video_dir;
use notify_rust::{Hint, Notification};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, info};
use tracing_subscriber::fmt::Subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional delay for the recording
    #[arg(short, long)]
    delay: Option<u64>,

    /// Optional output filename
    #[arg(short, long)]
    output: Option<PathBuf>,
}

// -video_size [width]x[height] -i [x],[y]
// 1846x1053+74+27
fn get_selection() -> Result<String> {
    let mut cmd = Command::new("slop");

    let cmd_output = cmd.output()?;

    let re = Regex::new(r"(?P<width>\d+)x(?P<height>\d+)\+(?P<x>\d+)\+(?P<y>\d+)")?;
    let cap = re
        .captures(str::from_utf8(&cmd_output.stdout)?)
        .context(format!("cannot match the slop {:?}", cmd_output))?;

    let width = cap
        .name("width")
        .context("missing width in output")?
        .as_str();
    let height = cap
        .name("height")
        .context("missing height in output")?
        .as_str();
    let x = cap.name("x").context("missing x in output")?.as_str();
    let y = cap.name("y").context("missing y in output")?.as_str();

    // -i :0.0+100,200
    Ok(format!(
        "-video_size {}x{} -i :0.0+{},{}",
        width, height, x, y
    ))
}

#[tracing::instrument]
pub fn generate_output() -> Result<PathBuf> {
    let datetime = Local::now();
    let filename_with_timestamp = datetime.to_rfc3339_opts(SecondsFormat::Secs, false);
    let mut video_dir = video_dir().context("Failed to get video_dir")?;
    video_dir.push(filename_with_timestamp);

    let video_dir = video_dir.with_extension("mkv");

    debug!("{:?}", video_dir);

    Ok(video_dir)
}

// ffmpeg -video_size 1024x768 -framerate 25 -f x11grab -i :0.0+100,200 output.mp4
fn create_ffmpeg_command(selection: String, output: &Path) -> Result<Command> {
    let mut res = Command::new("ffmpeg");
    res.args(&["-f", "x11grab"]);
    res.args(selection.split(' '));
    res.arg(output);

    Ok(dbg!(res))
}

/// Create notification to quickly open a file
fn create_stop_notification() -> Notification {
    let mut res = Notification::new();

    res.summary("Stop the recording")
        .action("stop", "Stop the recording")
        .hint(Hint::Resident(true));

    res
}

fn main() -> Result<()> {
    Subscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Hello, world!");

    let cli = Cli::parse();

    if let Some(delay) = cli.delay {
        sleep(Duration::from_secs(delay))
    }

    let output_file = cli.output.unwrap_or_else(|| generate_output().unwrap());

    debug!("{:?}", output_file);

    let selection = get_selection()?;

    let mut ffmpeg_command = create_ffmpeg_command(selection, &output_file)?;

    let mut ffmpeg_handle = ffmpeg_command.spawn()?;

    let notification = create_stop_notification();
    notification.show()?.wait_for_action(|_| {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(ffmpeg_handle.id() as i32),
            nix::sys::signal::Signal::SIGINT,
        )
        .expect("cannot send ctrl-c");
    });

    ffmpeg_handle.wait()?;

    debug!("{:?}", output_file);

    let mut open_notification = Notification::new();
    open_notification
        .summary("Open recording")
        .action("open", "Open recording")
        .hint(Hint::Resident(true));
    notification.show()?.wait_for_action(|_| {
        Command::new("mpv").arg(output_file).spawn().unwrap();
    });

    Ok(())
}
