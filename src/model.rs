#[derive(rocket::serde::Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
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

#[derive(rocket::serde::Serialize, Clone, Debug)]
pub struct Room {
    pub id: String,
    #[serde(rename = "hostId")]
    pub host_id: String,
    pub visibility: Visibility,
    pub state: RoomState,
    #[serde(rename = "maxUsers")]
    pub max_users: usize,
    #[serde(rename = "maxRounds")]
    pub max_rounds: u8,
}

impl Room {
    /// Create a new room with the given id, host id, and visibility.
    /// The room will be in `RoomState::Waiting` initially.
    pub fn new(id: String, host_id: String, visibility: Visibility) -> Self {
        Self {
            id,
            host_id,
            visibility,
            state: RoomState::Waiting,
            max_users: 8,
            max_rounds: 4,
        }
    }
}

#[derive(rocket::serde::Serialize, Clone, Debug)]
pub struct User {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing)]
    pub room_id: String,
    #[serde(skip_serializing)]
    pub has_drawn: bool,
}

impl User {
    pub fn new(id: String, display_name: String, room_id: String) -> Self {
        Self {
            id,
            display_name,
            room_id,
            has_drawn: false,
        }
    }
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
}

#[derive(rocket::serde::Serialize, Clone)]
pub struct HandshakePayload {
    pub user: User,
    pub room: Room,
    pub users_in_room: Vec<User>,
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

#[derive(rocket::serde::Serialize, Clone)]
pub struct ApiError {
    pub message: String,
}

impl ApiError {
    pub fn new(message: Option<String>) -> Self {
        Self {
            message: message.unwrap_or_else(|| "An error occurred".to_string()),
        }
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(
        self,
        _request: &'r rocket::Request<'_>,
    ) -> rocket::response::Result<'static> {
        let stringified_payload = serde_json::to_string(&self).map_err(|err| {
            rocket::error!("Failed to serialize ApiError: {:?}", err);

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
            4 => Ok(Self::StartGame),
            5 => Ok(Self::EndGame),
            6 => Ok(Self::NewRound),
            7 => Ok(Self::NewUserToDraw),
            8 => Ok(Self::PointerDown),
            9 => Ok(Self::PointerMove),
            10 => Ok(Self::PointerUp),
            11 => Ok(Self::ChangeColor),
            12 => Ok(Self::Tick),
            13 => Ok(Self::ResetRoom),
            14 => Ok(Self::NewHost),
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
            WebSocketEvents::StartGame => Ok(4),
            WebSocketEvents::EndGame => Ok(5),
            WebSocketEvents::NewRound => Ok(6),
            WebSocketEvents::NewUserToDraw => Ok(7),
            WebSocketEvents::PointerDown => Ok(8),
            WebSocketEvents::PointerMove => Ok(9),
            WebSocketEvents::PointerUp => Ok(10),
            WebSocketEvents::ChangeColor => Ok(11),
            WebSocketEvents::Tick => Ok(12),
            WebSocketEvents::ResetRoom => Ok(13),
            WebSocketEvents::NewHost => Ok(14),
        }
    }
}
