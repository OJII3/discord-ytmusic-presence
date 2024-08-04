mod playerctl;

use futures_util::future::join;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncompingBody, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use anyhow::bail;
use discord_sdk::activity::ActivityBuilder;
use discord_sdk::activity::Assets;
use discord_sdk::activity::Button;
use discord_sdk::wheel::UserState;
use discord_sdk::wheel::Wheel;
use discord_sdk::{AppId, Discord, DiscordApp};
use tokio;

use anyhow::Context as _;
use discord_sdk::Subscriptions;
use mpris::{PlayerFinder, ProgressTick};
use tracing::{error, info};

const APP_ID: AppId = 1267187960227565651;

#[derive(Deserialize, Debug)]
struct Music {
    music_url: String,
    thumbnail_url: String,
}

impl Clone for Music {
    fn clone(&self) -> Self {
        Music {
            music_url: self.music_url.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
        }
    }
}

static CURRENT_MUSIC: Lazy<Mutex<Music>> = Lazy::new(|| {
    let music = Mutex::new(Music {
        music_url: "https://open.spotify.com/track/7lPN2DXiMsVn7XUKtOW1CS?si=8b1".to_string(),
        thumbnail_url: "./media/ytmusic.png".to_string(),
    });
    music
});

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

    async fn handle_request(
        req: Request<IncompingBody>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        let whole_body = req.collect().await.unwrap().aggregate();
        // Decode as JSON...
        let mut data: serde_json::Value = serde_json::from_reader(whole_body.reader()).unwrap();
        // Change the JSON...
        match serde_json::from_value::<Music>(data.clone()) {
            Ok(music) => {
                println!("Music: {:?}", music);
                CURRENT_MUSIC.lock().unwrap().thumbnail_url = music.thumbnail_url;
            }
            Err(_) => {
                let current_music = CURRENT_MUSIC.lock().unwrap();
                data["music_url"] = serde_json::Value::String(current_music.music_url.clone());
                data["thumbnail_url"] =
                    serde_json::Value::String(current_music.thumbnail_url.clone());
            }
        }
        // And respond with the new JSON.
        Ok(Response::new(Full::new(Bytes::from("Hello World!"))))
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], 8477));
    let server = async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            let (tcp, _) = listener.accept().await.unwrap();

            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(tcp);

            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, service_fn(handle_request))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    };
    info!("Server started at http://{}", addr);

    // Create Playerctl instance

    let discord_process = async move {
        loop {
            let Ok(_player) = PlayerFinder::new().unwrap().find_active() else {
                continue;
            };
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
            let thumbnail_url = CURRENT_MUSIC.lock().unwrap().clone().thumbnail_url;

            let rich_presence = ActivityBuilder::default()
                .assets(Assets::default().large(thumbnail_url, Some("YT Music")))
                .state(format!("by {} - {} / {}", artist, time_elapsed, time_total))
                .details(title)
                .button(Button {
                    label: "Open".to_string(),
                    url: "https://open.spotify.com".to_string(),
                });

            info!(
                "updated activity: {:?}",
                discord.update_activity(rich_presence).await
            );

            // From others, presence is updated every 5 seconds, so no need to update it every second
            thread::sleep(Duration::from_secs(1));
        }
    };

    let _result = join(server, discord_process).await;

    Ok(())
}
