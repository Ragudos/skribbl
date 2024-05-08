use crate::utils;

#[derive(
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
}

#[derive(Clone)]
pub struct WordToDraw(pub String);

impl WordToDraw {
    pub fn get_three_words() -> [String; 3] {
        [
            utils::get_random_word().to_string(),
            utils::get_random_word().to_string(),
            utils::get_random_word().to_string(),
        ]
    }
}

#[derive(
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum PlayingState {
    PickingAWord {
        words_to_pick: [String; 3],
        /// When the user started picking a word.
        started_at: rocket::time::OffsetDateTime,
    },
    Drawing {
        current_word: String,
        started_at: rocket::time::OffsetDateTime,
    },
}

impl Default for PlayingState {
    fn default() -> Self {
        Self::PickingAWord {
            words_to_pick: WordToDraw::get_three_words(),
            started_at: rocket::time::OffsetDateTime::now_utc(),
        }
    }
}

#[derive(
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum RoomState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "playing")]
    Playing {
        #[serde(skip_serializing)]
        playing_state: PlayingState,
        #[serde(rename = "currentUserId")]
        current_user_id: String,
        #[serde(rename = "currentRound")]
        current_round: u8,
    },
    #[serde(rename = "finished")]
    Finished,
}

#[derive(
    rocket::serde::Serialize, rocket::serde::Deserialize, derive_builder::Builder, Clone,
)]
pub struct Room {
    pub id: String,
    #[serde(rename = "hostId")]
    pub host_id: String,
    #[builder(default = "Visibility::Public")]
    pub visibility: Visibility,
    #[builder(default = "RoomState::Waiting")]
    pub state: RoomState,
    #[builder(default = "8")]
    #[serde(rename = "maxUsers")]
    pub max_users: u8,
    #[builder(default = "4")]
    #[serde(rename = "maxRounds")]
    pub max_rounds: u8,
    #[builder(default = "1")]
    #[serde(skip_serializing)]
    pub amount_of_users: u8,
}

#[derive(rocket::serde::Serialize, derive_builder::Builder, Clone)]
pub struct User {
    pub id: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub room_id: String,
    #[builder(default = "false")]
    #[serde(skip_serializing)]
    pub has_drawn: bool,
}

#[derive(Clone, Default)]
pub struct GameState {
    pub rooms: std::sync::Arc<rocket::futures::lock::Mutex<Vec<Room>>>,
    pub users: std::sync::Arc<rocket::futures::lock::Mutex<Vec<User>>>,
}
