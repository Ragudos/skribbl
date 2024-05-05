use crate::utils;

#[derive(rocket::serde::Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(rocket::serde::Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(rocket::serde::Serialize, derive_builder::Builder, Clone, Debug)]
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
    #[builder(default)]
    pub amount_of_users: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

#[derive(rocket::serde::Serialize, derive_builder::Builder, Clone, Debug)]
pub struct User {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing)]
    pub room_id: String,
    #[serde(skip_serializing)]
    #[builder(default = "false")]
    pub has_drawn: bool,
    #[serde(skip_serializing)]
    #[builder(default = "ConnectionState::Connecting")]
    pub connection_state: ConnectionState,
}

#[derive(Clone, derive_builder::Builder)]
pub struct WebSocketMessage {
    pub user_id_to_exclude: Option<String>,
    pub room_id: String,
    pub message: ws::Message,
}

impl WebSocketMessage {
    pub fn new(
        user_id_to_exclude: Option<String>,
        room_id: String,
        message: ws::Message,
    ) -> Self {
        Self {
            user_id_to_exclude,
            room_id,
            message,
        }
    }
}

#[derive(Clone)]
pub struct WebSocketTick {
    pub room_id: String,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub rooms: std::sync::Arc<rocket::futures::lock::Mutex<Vec<Room>>>,
    pub users: std::sync::Arc<rocket::futures::lock::Mutex<Vec<User>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            rooms: std::sync::Arc::new(rocket::futures::lock::Mutex::new(Vec::new())),
            users: std::sync::Arc::new(rocket::futures::lock::Mutex::new(Vec::new())),
        }
    }
}

#[derive(rocket::FromFormField, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandshakeMode {
    #[field(value = "play")]
    Play,
    #[field(value = "create")]
    Create,
}

#[derive(rocket::FromForm)]
pub struct HandshakeData {
    #[field(name = "displayName", validate = len(3..=20))]
    pub display_name: String,
    /// The room id that the user wants to join.
    /// This is optional, but we don't use Option<String>
    /// since the field will include
    ///
    /// ```html
    /// <input type="text" name="roomId" value="{{roomId}}" />
    /// ```
    ///
    /// from the template, thus sending an empty string if it doesn't
    /// exist in the query params.
    #[field(name = "roomId")]
    pub room_id: String,
    pub mode: HandshakeMode,
}

#[derive(rocket::serde::Serialize, derive_builder::Builder, Clone)]
pub struct HandshakePayload {
    pub user: User,
    pub room: Room,
    #[serde(rename = "usersInRoom")]
    pub users_in_room: Vec<User>,
    #[serde(rename = "binaryProtocolVersion")]
    #[builder(default = "utils::consts::BINARY_PROTOCOL_VERSION")]
    pub binary_protocol_version: u8,
}

#[derive(Clone)]
pub enum ClientToServerEvents {
    StartGame,
    LeaveRoom,
    PointerDown,
    PointerMove,
    PointerUp,
    ChangeColor,
}

#[derive(Clone)]
pub enum ServerToClientEvents {
    Error { message: String },
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
    PointerMove,
    PointerUp,
    ChangeColor { color: String },
    Tick { time_left: u8 },
}

#[derive(Clone)]
pub enum WordToDraw {
    Word(String),
    ObfuscatedWord(String),
}
