use crate::content::song::UNKNOWN_ARTIST_STR;
use crate::{read_rwlock, write_rwlock};
use discord_rich_presence::activity::{
    Activity, ActivityType, Assets, StatusDisplayType, Timestamps,
};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APPLICATION_ID_STR: &str = "1470966026106830868";

static RPC_ACTION_TX: RwLock<Option<Sender<RPCAction>>> = RwLock::new(None);

// TODO: this doesnt really need to be a separate thread

#[derive(Clone, Debug)]
pub(super) struct SetSongData {
    pub(super) song_title: String,
    pub(super) song_artist: String,
    pub(super) song_duration: Option<Duration>,
}

#[derive(Debug)]
pub(super) enum RPCAction {
    Kill,
    SetSong(SetSongData),
    StopMusic,
    Resume,
    SetPlaylistName(String),
}

pub(super) fn discord_rpc_thread() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    **write_rwlock(&RPC_ACTION_TX) = Some(tx);

    let mut client = DiscordIpcClient::new(APPLICATION_ID_STR);

    let mut idle_activity = Activity::new()
        .details("Idling...")
        .activity_type(ActivityType::Listening)
        .status_display_type(StatusDisplayType::Details)
        .assets(Assets::new().small_image("napoleon_icon"));

    let mut activity = idle_activity.clone();

    client.connect()?;

    let mut use_idle_activity = true;

    client.set_activity(activity.clone()).ok();

    for action in rx.iter() {
        match action {
            RPCAction::Kill => {
                break;
            }

            RPCAction::SetSong(ss_data) => {
                activity = set_activity_to_song_data(activity, ss_data);
                use_idle_activity = false;
            }

            RPCAction::StopMusic => {
                use_idle_activity = true;
            }

            RPCAction::Resume => {
                use_idle_activity = false;
            }

            RPCAction::SetPlaylistName(playlist_name) => {
                idle_activity = idle_activity.state(format!("Browsing playlist {}", playlist_name));
            }
        }

        let activity_to_use = if use_idle_activity {
            idle_activity.clone()
        } else {
            activity.clone()
        };

        if client.set_activity(activity_to_use).is_err() {
            client.reconnect().ok();
        }
    }

    client.close()?;

    Ok(())
}

fn set_activity_to_song_data(activity: Activity, set_song_data: SetSongData) -> Activity {
    let SetSongData {
        song_title,
        song_artist,
        song_duration,
    } = set_song_data;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Invalid time")
        .as_millis() as i64;

    let mut timestamp = Timestamps::new().start(current_time);

    if let Some(song_duration) = song_duration {
        timestamp = timestamp.end(current_time + song_duration.as_millis() as i64)
    }

    let (state_string, details_prefix) = if song_artist != UNKNOWN_ARTIST_STR {
        (format!("By {}", song_artist), format!("{} - ", song_artist))
    } else {
        ("Jammin'".into(), String::new())
    };

    let details_string = format!("{}{}", details_prefix, song_title);

    activity
        .timestamps(timestamp)
        .state(state_string)
        .details(details_string)
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

pub fn set_rpc_playlist(playlist_name: String) {
    send_rpc_action(RPCAction::SetPlaylistName(playlist_name))
}
