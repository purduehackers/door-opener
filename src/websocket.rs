use std::{env, time::Duration};

use async_tungstenite::{
    tokio::connect_async,
    tungstenite::{Bytes, Message},
};
use futures::prelude::*;
use tokio::time::sleep;

use crate::camera::capture_photo;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
enum WebSocketMessage {
    Open,
    OpenAck,
    CapturePhoto,
    PhotoResult { data: String },
}

/// Websocket entry
///
/// # Panics
///
/// Will panic if there is no API key found
pub async fn ws_entry<F>(mut open: F)
where
    F: FnMut() + Send + 'static,
{
    loop {
        let (socket, _resp) =
            match connect_async("wss://api.purduehackers.com/phonebell/door-opener").await {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Failed to connect to API WebSocket: {e}");
                    return;
                }
            };

        let (mut write, mut read) = socket.split();

        write
            .send(Message::Text(
                env::var("DOOR_OPENER_API_KEY")
                    .expect("door opener API key")
                    .into(),
            ))
            .await
            .expect("write auth");

        loop {
            tokio::select! {
                () = sleep(Duration::from_secs(25)) => {
                    write.send(Message::Ping(Bytes::default())).await.expect("ping");
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(t))) => {
                            if let Ok(msg) = serde_json::from_str(t.as_ref()) {
                                match msg {
                                    WebSocketMessage::Open => {
                                        open();
                                        let res = write.send(Message::Text(serde_json::to_string(&WebSocketMessage::OpenAck).unwrap().into())).await;
                                        if let Err(e) = res {
                                        eprintln!("Failed to send open ack: {e:?}");
                                        }
                                    },
                                    WebSocketMessage::CapturePhoto => {
                                        let photostring = capture_photo();
                                        if let Ok(photostring) = photostring {
                                            let res = write.send(Message::Text(serde_json::to_string(&WebSocketMessage::PhotoResult { data: photostring }).unwrap().into())).await;
                                            if let Err(e) = res {
                                                eprintln!("Failed to send photo result: {e:?}");
                                            }
                                        } else {
                                            eprintln!("Failed to capture photo: {photostring:?}");
                                        }
                                    },
                                    WebSocketMessage::OpenAck | WebSocketMessage::PhotoResult { .. } => {}
                                }
                            }
                        }
                        Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                        Some(Err(e)) => eprintln!("Received err: {e:?}"),
                        None => break,
                        _ => eprintln!("Unsupported message received! {msg:?}"),
                    }
                }
            }
        }

        println!("WebSocket connection closed.");
    }
}
