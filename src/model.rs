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

impl Default for Visibility {
    fn default() -> Self {
        Self::Public
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
        #[serde(rename = "userToDraw")]
        user_to_draw: String,
        /// The word that the user is currently drawing.
        /// This field is not serialized to the client,
        /// since only the user drawing should know the word.
        /// We will hide this word from the other clients. For
        /// example, `current_word: "apple"` will be shown as
        /// `current_word: "____"`.
        #[serde(skip_serializing)]
        current_word: String,
        /// The time left for the user to draw the word.
        /// This field is not serialized to the client, since
        /// we will be sending this separately as a tick event.
        #[serde(skip_serializing)]
        time_left: u8,
        #[serde(skip_serializing)]
        current_round: u8,
    },
    #[serde(rename = "finished")]
    Finished,
}

impl Default for RoomState {
    fn default() -> Self {
        Self::Waiting
    }
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
    #[builder(default)]
    pub visibility: Visibility,
    #[builder(default)]
    pub state: RoomState,
    #[serde(rename = "maxUsers")]
    #[builder(default = "8")]
    pub max_users: usize,
    #[serde(rename = "maxRounds")]
    #[builder(default = "4")]
    pub max_rounds: u8,
    #[serde(skip_serializing)]
    #[builder(default = "1")]
    pub amount_of_users: usize,
}

#[derive(
    rocket::serde::Serialize,
    rocket::serde::Deserialize,
    derive_builder::Builder,
    Clone,
    Debug,
)]
pub struct User {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing)]
    pub room_id: String,
    #[serde(skip_serializing)]
    #[builder(default = "false")]
    pub has_drawn: bool,
}

#[derive(Clone)]
pub enum WebSocketMessageType {
    /// Send a message to everyone in the room including the sender.
    Everyone,
    /// Send a message to all users in the room except the sender.
    Broadcast { sender_id: String },
    /// Send a message to a specific user in the room.
    User(String),
}

#[derive(Clone, derive_builder::Builder)]
pub struct WebSocketMessage {
    pub r#type: WebSocketMessageType,
    pub room_id: String,
    pub message: ws::Message,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub rooms: std::sync::Arc<rocket::futures::lock::Mutex<Vec<Room>>>,
    pub users: std::sync::Arc<rocket::futures::lock::Mutex<Vec<User>>>,
}

/// Events received from the client stream;
#[derive(Clone)]
pub enum ClientToServerEvents {
    StartGame,
    PointerDown,
    PointerMove { direction: Direction },
    PointerUp,
    Message { message: String },
}

#[derive(Clone, rocket::serde::Serialize, rocket::serde::Deserialize)]
pub struct Direction {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub enum ServerToClientEvents {
    Error { message: String },
    ConnectError { message: String },
    UserJoined { user: User },
    UserLeft { user_id: String },
    StartGame,
    EndGame,
    ResetRoom,
    NewRound { round: u8, user_id_to_draw: String },
    NewUserToDraw { user_id: String },
    NewHost { user_id: String },
    NewWord { word: WordToDraw },
    PointerDown,
    PointerMove { direction: Direction },
    PointerUp,
    ChangeColor { color: String },
    Tick { time_left: u8 },
    SendUserInfo { user: User },
    SendRoomInfo { room: Room },
    SendUsersInRoomInfo { users: Vec<User> },
    Message { message: String },
}

#[derive(Clone)]
pub enum WordToDraw {
    Word(String),
    ObfuscatedWord(String),
}
