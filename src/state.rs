use crate::utils;

#[derive(
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
}

#[derive(Clone, Debug)]
pub struct WordToDraw(pub String);

impl WordToDraw {
    pub fn get_three_words() -> [String; 3] {
        let mut words: Vec<&str> = Vec::with_capacity(3);

        while words.len() != 3 {
            let word = utils::get_random_word();

            if words.contains(&word) {
                continue;
            }

            words.push(word);
        }

        [
            words.get(0).unwrap().to_string(),
            words.get(1).unwrap().to_string(),
            words.get(2).unwrap().to_string(),
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_words_to_draw_not_repeated() {
        for _ in 0..(utils::consts::WORDS.len() / 3) {
            let words = WordToDraw::get_three_words();
            assert_ne!(words[0], words[1]);
            assert_ne!(words[0], words[2]);
            assert_ne!(words[1], words[2]);
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
    Debug,
)]
pub enum PlayingState {
    #[serde(rename = "pickingAWord")]
    PickingAWord {
        #[serde(rename = "wordsToPick")]
        words_to_pick: [String; 3],
        #[serde(skip_serializing)]
        time_left: u8,
    },
    #[serde(rename = "drawing")]
    Drawing {
        #[serde(rename = "currentWord")]
        current_word: String,
        #[serde(skip_serializing)]
        time_left: u8,
    },
}

impl Default for PlayingState {
    fn default() -> Self {
        Self::PickingAWord {
            words_to_pick: WordToDraw::get_three_words(),
            time_left: utils::consts::PICK_WORD_TIME_LIMIT,
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
    Debug,
)]
pub enum RoomState {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "playing")]
    Playing {
        #[serde(rename = "playingState")]
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
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    derive_builder::Builder,
    Clone,
    Debug,
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

#[derive(rocket::serde::Serialize, derive_builder::Builder, Clone, Debug)]
pub struct User {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing)]
    pub room_id: String,
    #[builder(default = "false")]
    #[serde(skip_serializing)]
    pub has_drawn: bool,
    #[builder(default)]
    #[serde(skip_serializing)]
    pub score: u16,
}

#[derive(Clone)]
pub enum TickerCommand {
    Delete,
}

#[derive(Clone, derive_builder::Builder)]
pub struct TickerMsg {
    pub room_id: String,
    pub command: TickerCommand,
}

#[derive(Clone, Default)]
pub struct GameState {
    pub rooms: std::sync::Arc<rocket::futures::lock::Mutex<Vec<Room>>>,
    pub users: std::sync::Arc<rocket::futures::lock::Mutex<Vec<User>>>,
}
