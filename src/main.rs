mod handlers;
mod state;

use std::env;
use teloxide::prelude::*;
use dotenvy::dotenv;
use teloxide::dispatching::dialogue::InMemStorage;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting telegram bot...");

    // Load .env variables
    dotenv().ok();

    // Check variables
    let _token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");
    let admin_group_id_str = env::var("ADMIN_GROUP_ID").expect("ADMIN_GROUP_ID must be set");
    
    let admin_group_id = teloxide::types::ChatId(
        admin_group_id_str
            .parse::<i64>()
            .expect("ADMIN_GROUP_ID must be a valid integer"),
    );

    let bot = Bot::from_env();

    // Setup and start dispatcher
    Dispatcher::builder(bot, handlers::schema(admin_group_id))
        .dependencies(dptree::deps![InMemStorage::<state::State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
