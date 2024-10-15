
use dotenv::dotenv;
use std::env;

use teloxide::prelude::*;
use teloxide::types::ChatId;

use axum::{
    body::Bytes,
    extract::State, routing::post,
    Json, Router
};

use serde::Deserialize;

use std::sync::Arc;
use tokio::sync::Mutex;

use std::fs::File;
use std::io::Write;

#[derive(Clone)]
struct AppState {
    bot: Arc<Mutex<Bot>>,
    chat_id: ChatId,
}

impl AppState {
    fn new(bot: Bot, chat_id: ChatId) -> Self {
        Self {
            bot: Arc::new(Mutex::new(bot)),
            chat_id,
        }
    }

    async fn send_message(&self, message: String) {
        let bot = self.bot.lock().await;
        let chat_id = self.chat_id;

        println!("Sending {} to channel {}", message, self.chat_id);

        let res = bot.send_message(chat_id, message)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
        match res {
            Err(e) => println!("{}", e),
            _ => (),
        }
    }
}


#[derive(Debug, Deserialize)]
struct GHWebhookPayload {
    pusher: Pusher,
    commits: Vec<Commit>,
    repository: Repository,
}

#[derive(Debug, Deserialize)]
struct Pusher {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct Commit {
    id: String,
    message: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
}

#[tokio::main]
async fn main() {

    dotenv().ok();

    let chat_id =
        ChatId(env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID must be set!")
        .parse()
        .expect("TELEGRAM_CHAT_ID must be an i64!"));

    let tg_token =
        env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN must be set!");

    let addr : String =
        env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set!")
        .parse()
        .expect("SERVER_ADDR must be a valid address!");

    let bot = Bot::new(tg_token);
    let app_state = AppState::new(bot, chat_id);

    println!("Listening on {}", addr);

    app_state.send_message("Bot started".to_string()).await;

    let app = Router::new()
        .route("/webhook", post(github_webhook))
        .route("/save", post(save_webhook))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

async fn save_webhook(body : Bytes) {
    let content = String::from_utf8(body.to_vec()).unwrap();
    let mut file = File::create("saved_hook").unwrap();
    file.write_all(content.as_bytes()).unwrap();

}

async fn github_webhook(
    State(app_state): State<AppState>,
    Json(payload): Json<GHWebhookPayload>,
) {

    let message = format!(
        "New push to *{}* by {}:\n\n{}",
        payload.repository.full_name,
        payload.pusher.name,
        payload
            .commits
            .iter()
            .map(|commit| format!("• [{}]({})", commit.message, commit.url))
            .collect::<Vec<String>>()
            .join("\n")
    );

    println!("{}", &message);

    app_state.send_message(message).await;
}
