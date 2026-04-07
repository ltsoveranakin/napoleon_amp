use crate::content::song::song_data::Artist;
use crate::content::song::{Song, UNKNOWN_ARTIST_STR};
use crate::write_rwlock;
use discord_rich_presence::activity::{
    Activity, ActivityType, Assets, StatusDisplayType, Timestamps,
};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};


pub(crate) type ClientActivityArc = Arc<RwLock<ClientActivity>>;

const APPLICATION_ID_STR: &str = "1470966026106830868";

const UPDATE_INTERVAL: Duration = Duration::from_millis(1000);

// TODO: this doesnt really need to be a separate thread

#[derive(Clone, Debug)]
pub(super) struct SetSongData {
    pub(super) song_title: String,
    pub(super) song_artist: Artist,
    pub(super) song_duration: Option<Duration>,
}

struct ClientActivity {
    client: DiscordIpcClient,
    set_song_data: Option<SetSongData>,
    current_playlist_name: Option<String>,
}

pub(super) struct DiscordRPC {
    client_activity: ClientActivityArc,
    last_set_time: SystemTime,
    thread_handle: Option<JoinHandle<()>>,
}

impl DiscordRPC {
    pub(super) fn new() -> Self {
        // let latest_activity = Arc::new(RwLock::new(Self::idle_activity()));
        let mut client = DiscordIpcClient::new(APPLICATION_ID_STR);

        let _ = client.connect();

        Self {
            client_activity: Arc::new(RwLock::new(ClientActivity {
                client,
                set_song_data: None,
                current_playlist_name: None,
            })),
            last_set_time: SystemTime::now(),
            thread_handle: None,
        }
    }

    pub(crate) fn get_activity_from_song_data(set_song_data: Option<SetSongData>) -> Activity<'static> {
        let default_activity = Self::idle_activity();

        if set_song_data.is_none() {
            return default_activity;
        }

        let SetSongData {
            song_title,
            song_artist,
            song_duration,
        } = set_song_data.unwrap();

        let main_artist = song_artist.main_artist();

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Invalid time")
            .as_millis() as i64;

        let mut timestamp = Timestamps::new().start(current_time);

        if let Some(song_duration) = song_duration {
            timestamp = timestamp.end(current_time + song_duration.as_millis() as i64)
        }

        let (state_string, details_prefix) = if main_artist != UNKNOWN_ARTIST_STR {
            (
                format!("By {}", song_artist.full_artist_string),
                format!("{} - ", main_artist),
            )
        } else {
            (String::new(), String::new())
        };

        let details_string = format!("{}{}", details_prefix, song_title);

        let activity = Self::idle_activity().timestamps(timestamp)
            .state(state_string)
            .details(details_string);

        activity
    }

    pub(crate) fn set_song_data(&mut self, song: Option<&Song>) {
        let set_song_data = song.map(|song| {
            let song_data = **song.get_song_data();

            SetSongData {
                song_title: song_data.title,
                song_artist: song_data.artist,
                song_duration:
            }
        })

        let spawn_thread = SystemTime::now().duration_since(self.last_set_time).is_ok_and(|duration_since| duration_since < UPDATE_INTERVAL);

        if spawn_thread {
            if self.thread_handle.is_none() {
                let client_activity = Arc::clone(&self.client_activity);

                self.thread_handle = Some(spawn(move || {
                    sleep(UPDATE_INTERVAL);

                    let mut client_activity = write_rwlock(&client_activity);
                    let activity = Self::get_activity_from_song_data(set_song_data);

                    Self::send_activity(&mut client_activity.client, activity)
                }));
            }
        } else {
            let mut client_activity = write_rwlock(&self.client_activity);
            let activity = Self::get_activity_from_song_data(set_song_data);

            Self::send_activity(&mut client_activity.client, activity)
        }
    }

    pub fn set_playlist(&mut self, playlist_name: String) {
        let mut client_activity = write_rwlock(&self.client_activity);

        client_activity.current_playlist_name = Some(playlist_name);
    }

    fn idle_activity() -> Activity<'static> {
        Activity::new()
            .details("Idling...")
            .activity_type(ActivityType::Listening)
            .status_display_type(StatusDisplayType::Details)
            .assets(Assets::new().small_image("napoleon_icon"))
    }

    fn send_activity(client: &mut DiscordIpcClient, activity: Activity) {
        if client.set_activity(activity).is_err() {
            let _ = client.reconnect();
        }
    }
}

// #[derive(Debug)]
// pub(super) enum RPCAction {
//     Kill,
//     SetSong(SetSongData),
//     StopMusic,
//     Resume,
//     SetPlaylistName(String),
// }
//
// pub(super) fn send_rpc_action(action: RPCAction) {
//     // let mut kill_sender = false;
//     //
//     // {
//     //     let sender_opt = read_rwlock(&RPC_ACTION_TX);
//     //
//     //     if let Some(tx) = &**sender_opt {
//     //         if tx.send(action).is_err() {
//     //             kill_sender = true;
//     //         }
//     //     }
//     // }
//
//     let mut client = DiscordIpcClient::new(APPLICATION_ID_STR);
//
//     let mut idle_activity = Activity::new()
//         .details("Idling...")
//         .activity_type(ActivityType::Listening)
//         .status_display_type(StatusDisplayType::Details)
//         .assets(Assets::new().small_image("napoleon_icon"));
//
//     let mut activity = idle_activity.clone();
//
//     client.connect()?;
//
//     let mut use_idle_activity = true;
//
//     // client.set_activity(activity.clone()).ok();
//
//
//     match action {
//         RPCAction::Kill => {
//             break;
//         }
//
//         RPCAction::SetSong(ss_data) => {
//             activity = set_activity_to_song_data(activity, ss_data);
//             use_idle_activity = false;
//         }
//
//         RPCAction::StopMusic => {
//             use_idle_activity = true;
//         }
//
//         RPCAction::Resume => {
//             use_idle_activity = false;
//         }
//
//         RPCAction::SetPlaylistName(playlist_name) => {
//             idle_activity = idle_activity.state(format!("Browsing playlist {}", playlist_name));
//         }
//     }
//
//     let activity_to_use = if use_idle_activity {
//         idle_activity.clone()
//     } else {
//         activity.clone()
//     };
//
//     if client.set_activity(activity_to_use).is_err() {
//         client.reconnect().ok();
//     }
//
//
//     client.close()?;
//
//     if kill_sender {
//         write_rwlock(&RPC_ACTION_TX).take();
//     }
// }
//
// pub fn set_rpc_playlist(playlist_name: String) {
//     send_rpc_action(RPCAction::SetPlaylistName(playlist_name))
// }
