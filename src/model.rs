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
}

#[derive(Clone)]
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

impl<'r> rocket::response::Responder<'r, 'static> for HandshakePayload {
    fn respond_to(
        self,
        _request: &'r rocket::Request<'_>,
    ) -> rocket::response::Result<'static> {
        let stringified_payload = serde_json::to_string(&self).map_err(|err| {
            rocket::error!("Failed to serialize HandshakePayload: {:?}", err);

            rocket::http::Status::InternalServerError
        })?;

        rocket::response::Response::build()
            .header(rocket::http::ContentType::JSON)
            .sized_body(
                stringified_payload.len(),
                std::io::Cursor::new(stringified_payload),
            )
            .ok()
    }
}

/// All payloads in these events
/// are in a vector of bytes [`u8`].
#[derive(Clone)]
pub enum WebSocketEvents {
    Error,
    /// Broadcasted to all clients
    /// in a room when a
    /// user has joined the room, except the latter.
    /// The payload is as follows:
    ///
    /// ```rust
    /// vec![
    ///     BINARY_PROTOCOL_VERSION,
    ///     WebSocketEvents::UserJoined as u8,
    ///     // How long the [`UserLength`] is
    ///     // since we might have a string
    ///     // with a length of more than 255.
    ///     // For example, if the length of the
    ///     // string is 300, this will have:
    ///     // 300 / 255 = 1.171875, which will
    ///     // be rounded up to 2 since it
    ///     // takes 2 bytes of space. Thus, the
    ///     // `UserLengthSpace` will be 2.
    ///     UserLengthSpace as u8,
    ///     // The length of the serialized
    ///     // [`User`] struct's vector of bytes.
    ///     // Can be more than 1 byte.
    ///     UserLength as u8,
    ///     // The serialized
    ///     // [`User`] struct turned into
    ///     // a vector of bytes.
    ///     User { ... }
    /// ]
    /// ```
    UserJoined,
    UserLeft,
    StartGame,
    EndGame,
    NewRound,
    NewUserToDraw,
    PointerDown,
    PointerMove,
    PointerUp,
    ChangeColor,
    Tick,
    ResetRoom,
    NewHost,
}

impl TryFrom<u8> for WebSocketEvents {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: u8) -> Result<Self, <WebSocketEvents as TryFrom<u8>>::Error> {
        match value {
            0 => Ok(Self::Error),
            1 => Ok(Self::UserJoined),
            2 => Ok(Self::UserLeft),
            3 => Ok(Self::StartGame),
            4 => Ok(Self::EndGame),
            5 => Ok(Self::NewRound),
            6 => Ok(Self::NewUserToDraw),
            7 => Ok(Self::PointerDown),
            8 => Ok(Self::PointerMove),
            9 => Ok(Self::PointerUp),
            10 => Ok(Self::ChangeColor),
            11 => Ok(Self::Tick),
            12 => Ok(Self::ResetRoom),
            13 => Ok(Self::NewHost),
            _ => Err("Invalid WebSocketEvents payload".into()),
        }
    }
}

impl TryFrom<WebSocketEvents> for u8 {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: WebSocketEvents,
    ) -> Result<Self, <u8 as TryFrom<WebSocketEvents>>::Error> {
        match value {
            WebSocketEvents::Error => Ok(0),
            WebSocketEvents::UserJoined => Ok(1),
            WebSocketEvents::UserLeft => Ok(2),
            WebSocketEvents::StartGame => Ok(3),
            WebSocketEvents::EndGame => Ok(4),
            WebSocketEvents::NewRound => Ok(5),
            WebSocketEvents::NewUserToDraw => Ok(6),
            WebSocketEvents::PointerDown => Ok(7),
            WebSocketEvents::PointerMove => Ok(8),
            WebSocketEvents::PointerUp => Ok(9),
            WebSocketEvents::ChangeColor => Ok(10),
            WebSocketEvents::Tick => Ok(11),
            WebSocketEvents::ResetRoom => Ok(12),
            WebSocketEvents::NewHost => Ok(13),
        }
    }
}
