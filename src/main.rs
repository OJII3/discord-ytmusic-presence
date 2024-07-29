use discord_presence::{Client, Event};
use std::thread::sleep;
mod playerctl;

use mpris::{PlayerFinder, ProgressTick};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Create a new Discord RPC client with the application ID of the Discord app
    let mut drpc = Client::new(1267187960227565651);

    // Set the handlers for the events
    drpc.on_ready(|_ctx| {
        println!("ready?");
    })
    .persist();

    drpc.on_activity_join_request(|ctx| {
        println!("Join request: {:?}", ctx.event);
    })
    .persist();

    drpc.on_activity_join(|ctx| {
        println!("Joined: {:?}", ctx.event);
    })
    .persist();

    drpc.on_activity_spectate(|ctx| {
        println!("Spectate: {:?}", ctx.event);
    })
    .persist();

    // Start the RPC client
    drpc.start();
    drpc.block_until_event(Event::Ready)
        .expect("Failed to start RPC client");

    assert!(Client::is_ready());

    // Create Playerctl instance

    loop {
        let player = PlayerFinder::new().unwrap().find_active().unwrap();
        let identity = player.identity();
        println!("Found player: {}", identity);
        let mut progress_tracker = player.track_progress(100).unwrap();
        let ProgressTick { progress, .. } = progress_tracker.tick();

        if let Ok(_metadata) = player.get_metadata() {
            playerctl::print_playback_status(progress);
            playerctl::print_shuffle_status(progress);
            playerctl::print_loop_status(progress);
            let artist = playerctl::get_artist(progress.metadata());
            let title = playerctl::get_title(progress.metadata());
            let time_elapsed = playerctl::get_time(Some(progress.position()));
            let time_total = playerctl::get_time(progress.length());
            let thumbnail = playerctl::get_thumbnail(progress.metadata());

            // Set the activity
            drpc.set_activity(|act| {
                return act
                    .assets(|asset| asset.large_image(thumbnail).large_text("YT Music"))
                    .state(format!("by {} - {} / {}", artist, time_elapsed, time_total))
                    .details(format!("{}", title))
                    .append_buttons(|button| button.label("Open").url("https://open.spotify.com"));
            })
            .expect("Failed to set activity");

            sleep(std::time::Duration::from_secs(1));
        }
    }
}
