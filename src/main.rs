
use dotenv::dotenv;
use std::env;

use teloxide::prelude::*;
use teloxide::types::{ChatId, LinkPreviewOptions};
use teloxide::utils::markdown::escape;

use axum::{
    extract::State, routing::post,
    Json, Router
};

use serde::Deserialize;

use std::sync::Arc;
use tokio::sync::Mutex;

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

        let mut message = bot.send_message(chat_id, message)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2);

        message.link_preview_options = Some(LinkPreviewOptions {
            is_disabled: true,
            url: None,
            prefer_small_media: false,
            prefer_large_media: false,
            show_above_text: false,
        });



        match message.await {
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
    sender: Sender,
}

#[derive(Debug, Deserialize)]
struct Sender {
    login: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct Pusher {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Commit {
    message: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
    html_url: String,
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

    let app = Router::new()
        .route("/webhook", post(github_webhook))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn format_link(link_text: &String, link_url: &String) -> String {
    let escaped_text = escape(link_text);
    let escaped_url = escape(link_url);
    format!("[{escaped_text}]({escaped_url})")
}

async fn github_webhook(
    State(app_state): State<AppState>,
    Json(payload): Json<GHWebhookPayload>,
) {

    let link_to_repo = format_link(&payload.repository.full_name, &payload.repository.html_url);

    let link_to_sender = format_link(&payload.sender.login, &payload.sender.html_url);

    let formatted_commits =
        payload
        .commits
        .iter()
        .map(|commit| format!("• {}", format_link(&commit.message, &commit.url)))
        .collect::<Vec<String>>()
        .join("\n");

    let message = format!( "New push to {link_to_repo} by {link_to_sender}:\n\n{formatted_commits}");

    println!("{}", &message);

    app_state.send_message(message).await;
}
