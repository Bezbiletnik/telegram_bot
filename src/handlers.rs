use crate::state::{State, FormState};
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::ChatId,
    utils::command::BotCommands,
};
use std::error::Error;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "start the process")]
    Start,
    #[command(description = "cancel the process")]
    Cancel,
}

pub fn schema(admin_group_id: ChatId) -> UpdateHandler<Box<dyn Error + Send + Sync + 'static>> {
    use dptree::case;

    // The user flow is only for private chats
    let user_flow = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_private())
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::Cancel].endpoint(cancel))
        )
        .branch(case![State::Start].endpoint(start))
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(case![State::ReceiveProjectName(form)].endpoint(receive_project_name))
        .branch(case![State::ReceiveContactInfo(form)].endpoint(receive_contact_info))
        .branch(case![State::ReceiveQuestion(form)].endpoint(
            move |bot: Bot, dialogue: MyDialogue, msg: Message, form: FormState| {
                receive_question(bot, dialogue, msg, form, admin_group_id)
            }
        ));

    // Admin flow is only for the admin group
    let admin_flow = Update::filter_message()
        .filter(move |msg: Message| msg.chat.id == admin_group_id)
        .endpoint(admin_reply_handler);

    dptree::entry()
        .branch(user_flow)
        .branch(admin_flow)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Welcome! Let's get your contact request started.\n\nPlease enter your Full Name:")
        .await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Process cancelled. You can type /start to begin again.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Please enter your Project Name:").await?;
            let mut form = FormState::default();
            form.full_name = text.into();
            dialogue.update(State::ReceiveProjectName(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send text.").await?;
        }
    }
    Ok(())
}

async fn receive_project_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut form: FormState,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Please enter your Contact Info (Email and/or Phone number):").await?;
            form.project_name = text.into();
            dialogue.update(State::ReceiveContactInfo(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send text.").await?;
        }
    }
    Ok(())
}

async fn receive_contact_info(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut form: FormState,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Finally, what is your question or message?").await?;
            form.contact_info = text.into();
            dialogue.update(State::ReceiveQuestion(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send text.").await?;
        }
    }
    Ok(())
}

async fn receive_question(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    form: FormState,
    admin_group_id: ChatId,
) -> HandlerResult {
    match msg.text() {
        Some(question) => {
            bot.send_message(msg.chat.id, "Thank you! We have received your request and our admins will get back to you here shortly.").await?;
            
            // Format the ticket
            let ticket = format!(
                "🚨 **New Support Request**\n\
                 **ID:** {}\n\
                 **Name:** {}\n\
                 **Project:** {}\n\
                 **Contact:** {}\n\
                 **Question:** {}",
                msg.chat.id, form.full_name, form.project_name, form.contact_info, question
            );
            
            // Send to Admin Group
            bot.send_message(admin_group_id, ticket).await?;
            
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send text.").await?;
        }
    }
    Ok(())
}

async fn admin_reply_handler(bot: Bot, msg: Message) -> HandlerResult {
    if let Some(reply_to) = msg.reply_to_message() {
        if let Some(text) = reply_to.text() {
            // Very simple parser to extract ID
            // We look for "**ID:** 123456789"
            if let Some(id_str) = text.lines().find(|l| l.starts_with("**ID:** ")) {
                let id_str = id_str.trim_start_matches("**ID:** ");
                if let Ok(user_id) = id_str.parse::<i64>() {
                    let user_chat_id = ChatId(user_id);
                    if let Some(admin_reply_text) = msg.text() {
                        let final_message = format!("🧑‍💻 **Support Team Reply:**\n\n{}", admin_reply_text);
                        // Send the reply back to the user
                        if let Err(e) = bot.send_message(user_chat_id, final_message).await {
                            log::error!("Failed to send reply to user {}: {}", user_id, e);
                            bot.send_message(msg.chat.id, format!("Failed to deliver message to user: {}", e)).await?;
                        } else {
                            // React or confirm success
                            bot.send_message(msg.chat.id, "✅ Reply sent successfully.").await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
