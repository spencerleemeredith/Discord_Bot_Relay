use futures_util::SinkExt;
use poise::serenity_prelude::{self as serenity};
use std::env;
use tokio::sync::Mutex;
use serde::Serialize;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
use actix_web::{get, App, HttpServer, Responder, Result as ActixResult};
use actix_files::NamedFile;
use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use thiserror::Error;

mod websocket_server;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Environment variable error: {0}")]
    EnvError(#[from] env::VarError),
    #[error("Channel ID parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Discord client error: {0}")]
    DiscordError(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

type BotResult<T> = Result<T, BotError>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    bridge_channel_id: serenity::ChannelId,
    ws_sender: Arc<Mutex<Option<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
}

#[derive(Debug, Serialize)]
struct WebSocketMessage {
    message_type: String,
    author: String,
    content: String,
    timestamp: i64,
    message_id: String,
    reactions: Option<Vec<ReactionInfo>>,
}

#[derive(Debug, Serialize)]
struct ReactionInfo {
    emoji: String,
    count: i32,
    message_id: String,
}

#[tokio::main]
async fn main() -> BotResult<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Load .env file
    dotenv::dotenv().ok();

    // Get environment variables with proper error handling
    let token = env::var("DISCORD_TOKEN")?;
    let channel_id = env::var("BRIDGE_CHANNEL_ID")?;
    let bridge_channel_id = serenity::ChannelId::new(channel_id.parse()?);

    let ws_sender = Arc::new(Mutex::new(None));
    let ws_sender_clone = ws_sender.clone();

    let data = Data {
        bridge_channel_id,
        ws_sender,
    };

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS;

    // Start the WebSocket server
    let ws_server = tokio::spawn(websocket_server::run_websocket_server(ws_sender_clone));

    // Start the web server
    let web_server = HttpServer::new(move || {
        App::new()
            .service(index)
    })
    .bind(("127.0.0.1", 8081))
    .map_err(|e| BotError::FileError(e))?
    .run();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![],
                event_handler: |ctx, event, framework, data| {
                    Box::pin(event_handler(ctx, event, framework, data))
                },
                ..Default::default()
            })
            .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(data) }))
            .build()
        )
        .await
        .map_err(|e| BotError::DiscordError(e.to_string()))?;

    // Run both the Discord bot and servers
    tokio::select! {
        result = client.start() => {
            if let Err(e) = result {
                eprintln!("Discord bot error: {}", e);
            }
            println!("Discord bot finished");
        },
        _ = ws_server => println!("WebSocket server finished"),
        _ = web_server => println!("Web server finished"),
    }

    Ok(())
}

#[get("/")]
async fn index() -> ActixResult<impl Responder> {
    Ok(NamedFile::open_async("./static/index.html").await?)
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, BotError>,
    data: &Data,
) -> Result<(), BotError> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            if new_message.channel_id == data.bridge_channel_id {
                let ws_msg = WebSocketMessage {
                    message_type: "message".to_string(),
                    author: new_message.author.name.clone(),
                    content: new_message.content.clone(),
                    timestamp: new_message.timestamp.unix_timestamp(),
                    message_id: new_message.id.to_string(),
                    reactions: None,
                };
                
                let msg_json = serde_json::to_string(&ws_msg)?;
                if let Some(ref mut sender) = *data.ws_sender.lock().await {
                    if let Err(e) = sender.send(Message::Text(msg_json)).await {
                        eprintln!("Failed to send WebSocket message: {}", e);
                    }
                }
            }
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            if add_reaction.channel_id == data.bridge_channel_id {
                if let Ok(message) = add_reaction.message(&ctx.http).await {
                    let reactions: Vec<ReactionInfo> = message
                        .reactions
                        .iter()
                        .map(|r| ReactionInfo {
                            emoji: r.reaction_type.to_string(),
                            count: r.count as i32,
                            message_id: message.id.to_string(),
                        })
                        .collect();

                    let ws_msg = WebSocketMessage {
                        message_type: "reaction".to_string(),
                        author: "".to_string(),
                        content: "".to_string(),
                        timestamp: message.timestamp.unix_timestamp(),
                        message_id: message.id.to_string(),
                        reactions: Some(reactions),
                    };

                    let msg_json = serde_json::to_string(&ws_msg)?;
                    if let Some(ref mut sender) = *data.ws_sender.lock().await {
                        if let Err(e) = sender.send(Message::Text(msg_json)).await {
                            eprintln!("Failed to send WebSocket message: {}", e);
                        }
                    }
                }
            }
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            if removed_reaction.channel_id == data.bridge_channel_id {
                if let Ok(message) = removed_reaction.message(&ctx.http).await {
                    let reactions: Vec<ReactionInfo> = message
                        .reactions
                        .iter()
                        .map(|r| ReactionInfo {
                            emoji: r.reaction_type.to_string(),
                            count: r.count as i32,
                            message_id: message.id.to_string(),
                        })
                        .collect();

                    let ws_msg = WebSocketMessage {
                        message_type: "reaction".to_string(),
                        author: "".to_string(),
                        content: "".to_string(),
                        timestamp: message.timestamp.unix_timestamp(),
                        message_id: message.id.to_string(),
                        reactions: Some(reactions),
                    };

                    let msg_json = serde_json::to_string(&ws_msg)?;
                    if let Some(ref mut sender) = *data.ws_sender.lock().await {
                        if let Err(e) = sender.send(Message::Text(msg_json)).await {
                            eprintln!("Failed to send WebSocket message: {}", e);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
