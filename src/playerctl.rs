use std::fs::{metadata, File};
use std::io;
use std::io::Read;
use std::time::Duration;

use mpris::{LoopStatus, Metadata, PlaybackStatus, Progress};

pub fn get_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let whole_hours = secs / (60 * 60);

    let secs = secs - whole_hours * 60 * 60;
    let whole_minutes = secs / 60;

    let secs = secs - whole_minutes * 60;

    return format!("{:02}:{:02}", whole_minutes, secs);
}

pub fn get_time(duration: Option<Duration>) -> String {
    match duration {
        Some(duration) => get_duration(duration),
        None => return format!("??:??:??"),
    }
}

pub fn get_artist(metadata: &Metadata) -> String {
    if let Some(artists) = metadata.artists() {
        if !artists.is_empty() {
            return format!("{}", artists.join(" + "));
        }
    }

    format!("Unknown artist")
}

pub fn get_title(metadata: &Metadata) -> String {
    format!("{}", metadata.title().unwrap_or("Unknown title"))
}

pub fn print_playback_status(progress: &Progress) -> String {
    match progress.playback_status() {
        PlaybackStatus::Playing => format!("▶"),
        PlaybackStatus::Paused => format!("▮▮"),
        PlaybackStatus::Stopped => format!("◼"),
    }
}

pub fn print_shuffle_status(progress: &Progress) -> String {
    if progress.shuffle() {
        format!("🔀")
    } else {
        format!(" ")
    }
}

pub fn print_loop_status(progress: &Progress) -> String {
    match progress.loop_status() {
        LoopStatus::None => return format!(" "),
        LoopStatus::Track => return format!("🔂"),
        LoopStatus::Playlist => return format!("🔁"),
    }
}

pub fn get_thumbnail(metadata: &Metadata) -> String {
    if let Some(thumbnail) = metadata.art_url() {
        format!("{}", thumbnail)
    } else {
        format!("/")
    }
}
