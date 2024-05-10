use rocket::futures::SinkExt;

use crate::{events, state};

pub async fn create_websocket_writer(
    room_id: String,
    user_id: String,
    game_state: &rocket::State<state::GameState>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
    server_messages: &rocket::State<
        rocket::tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server_messages_rx = server_messages.subscribe();

    while let Some(server_message) = server_messages_rx.recv().await.ok() {
        match server_message.r#type {
            events::WebSocketMessageType::Everyone => {
                if room_id != server_message.room_id {
                    continue;
                }
            }
            events::WebSocketMessageType::Broadcast { sender_id } => {
                if sender_id == user_id || room_id != server_message.room_id {
                    continue;
                }
            }
            events::WebSocketMessageType::User { receiver_id } => {
                if receiver_id != user_id {
                    continue;
                }
            }
        }

        sink.lock()
            .await
            .send(server_message.message)
            .await?;
    }

    Ok(())
}
