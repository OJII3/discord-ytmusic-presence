mod playerctl;

use std::time::Duration;
use std::time::SystemTime;

use anyhow::bail;
use discord_sdk::activity::ActivityBuilder;
use discord_sdk::activity::Assets;
use discord_sdk::activity::Button;
// use discord_presence::{Client, Event};
// use std::thread::sleep;
use discord_sdk::wheel::UserState;
use discord_sdk::wheel::Wheel;
use discord_sdk::AppId;
use discord_sdk::Discord;
use discord_sdk::DiscordApp;

use anyhow::Context as _;
use discord_sdk::Subscriptions;
use mpris::{PlayerFinder, ProgressTick};
use tokio::time::sleep;
use tracing::{error, info};

const APP_ID: AppId = 1267187960227565651;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (wheel, handler) = Wheel::new(Box::new(|err| {
        error!(error = ?err, "encountered an error");
    }));

    let mut user = wheel.user();

    let discord = Discord::new(
        DiscordApp::PlainId(APP_ID),
        Subscriptions::ACTIVITY,
        Box::new(handler),
    )
    .context("unable to create discord client")?;

    tracing::info!("waiting for handshake...");
    user.0.changed().await?;

    let user = match &*user.0.borrow() {
        UserState::Connected(user) => user.clone(),
        UserState::Disconnected(err) => bail!("failed to connect to Discord: {}", err),
    };

    info!("connected to Discord, local user is {:#?}", user);

    let mut activity_events = wheel.activity();

    tokio::task::spawn(async move {
        while let Ok(event) = activity_events.0.recv().await {
            tracing::info!("received activity event: {:#?}", event);
        }
    });

    // Create Playerctl instance
    loop {
        let player = PlayerFinder::new().unwrap().find_active().unwrap();
        let identity = player.identity();
        println!("Found player: {}", identity);
        let mut progress_tracker = player.track_progress(1000).unwrap();
        let ProgressTick { progress, .. } = progress_tracker.tick();

        let Ok(_metadata) = player.get_metadata() else {
            continue;
        };
        playerctl::print_playback_status(progress);
        playerctl::print_shuffle_status(progress);
        playerctl::print_loop_status(progress);
        let artist = playerctl::get_artist(progress.metadata());
        let title = playerctl::get_title(progress.metadata());
        let time_elapsed = playerctl::get_time(Some(progress.position()));
        let time_total = playerctl::get_time(progress.length());
        let thumbnail = playerctl::get_thumbnail(progress.metadata());

        // Set the activity
        //drpc.set_activity(|act| {
        //    return act
        //        .assets(|asset| asset.large_image(thumbnail).large_text("YT Music"))
        //        .state(format!("by {} - {} / {}", artist, time_elapsed, time_total))
        //        .details(format!("{}", title))
        //        .append_buttons(|button| button.label("Open").url("https://open.spotify.com"));
        //})
        //.expect("Failed to set activity");

        let rich_presence = ActivityBuilder::default()
            .assets(Assets::default().large(thumbnail, Some("YT Music")))
            .state(format!("by {} - {} / {}", artist, time_elapsed, time_total))
            .details(title)
            .button(Button {
                label: "Open".to_string(),
                url: "https://open.spotify.com".to_string(),
            });

        info!(
            "updated activity: {:?}",
            discord.update_activity(rich_presence).await?
        );

        sleep(Duration::from_secs(1)).await;
    }
}
