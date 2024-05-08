use rocket::futures::StreamExt;
use rocket::tokio;

use crate::{events, state};

pub async fn create_websocket_reader(
    room_id: String,
    user_id: String,
    game_events: &rocket::State<state::GameState>,
    mut stream: rocket::futures::stream::SplitStream<ws::stream::DuplexStream>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(message) = stream.next().await {
        let message = message?;

        match message {
            ws::Message::Binary(data) => {
                let borrowed = &data;
                let event_type = borrowed.try_into()?;

                match event_type {
                    events::ClientToServerEvents::StartGame => todo!(),
                    _ => {}
                }

                // Put this outside so we don't have to clone the data
                let _ = events::WebSocketMessageBuilder::default()
                    .r#type(events::WebSocketMessageType::Broadcast {
                        sender_id: user_id.clone(),
                    })
                    .room_id(room_id.clone())
                    .message(ws::Message::Binary(data))
                    .build()?
                    .send(server_messages);
            }
            ws::Message::Close(close_frame) => {
                if let Some(close_frame) = &close_frame {
                    println!("Closing connection: {:#?}", close_frame);
                }

                break;
            }
            _ => {}
        }
    }

    Ok(())
}
