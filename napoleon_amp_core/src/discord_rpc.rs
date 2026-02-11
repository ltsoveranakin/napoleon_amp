use crate::{read_rwlock, write_rwlock};
use discord_rich_presence::activity::{Activity, ActivityType, StatusDisplayType, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APPLICATION_ID_STR: &str = "1470966026106830868";

static RPC_ACTION_TX: RwLock<Option<Sender<RPCAction>>> = RwLock::new(None);

// TODO: this doesnt really need to be a separate thread, implement using statics and functions to call the statics

pub(super) enum RPCAction {
    Kill,
    ChangeSong {
        song_title: String,
        song_artist: String,
        song_duration: Option<Duration>,
    },
}

pub(super) fn discord_rpc_thread() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    **write_rwlock(&RPC_ACTION_TX) = Some(tx);

    let mut client = DiscordIpcClient::new(APPLICATION_ID_STR);
    let mut activity = Activity::new()
        .state("Idling...")
        .activity_type(ActivityType::Listening)
        .status_display_type(StatusDisplayType::Details);

    client.connect()?;

    client
        .set_activity(activity.clone())
        .expect("Unable to set activity");

    loop {
        if let Ok(action) = rx.recv() {
            match action {
                RPCAction::Kill => {
                    break;
                }

                RPCAction::ChangeSong {
                    song_title,
                    song_artist,
                    song_duration,
                } => {
                    println!("rcv song {:?}", song_title);
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Invalid time")
                        .as_millis() as i64;

                    let mut timestamp = Timestamps::new().start(current_time);

                    if let Some(song_duration) = song_duration {
                        timestamp = timestamp.end(current_time + song_duration.as_millis() as i64)
                    }

                    activity = activity
                        .timestamps(timestamp)
                        .state(format!("By {}", song_artist))
                        .details(format!("Listening to {}", song_title));
                }
            }

            client
                .set_activity(activity.clone())
                .expect("Unable to set activity");
            println!("set activity");
        } else {
            println!("rpc channel hung up");
            // channel hung up
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }

    client.close()?;

    Ok(())
}

pub(super) fn send_rpc_action(action: RPCAction) {
    let mut kill_sender = false;

    {
        let sender_opt = read_rwlock(&RPC_ACTION_TX);

        if let Some(tx) = &**sender_opt {
            if tx.send(action).is_err() {
                kill_sender = true;
            }
        }
    }

    if kill_sender {
        write_rwlock(&RPC_ACTION_TX).take();
    }
}
