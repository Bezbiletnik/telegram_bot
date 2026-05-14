use crate::state::{State, FormState};
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use std::error::Error;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Поддерживаемые команды / Қолдау көрсетілетін пәрмендер:")]
pub enum Command {
    #[command(description = "показать приветствие / сәлемдесуді көрсету")]
    Start,
    #[command(description = "задать вопрос / сұрақ қою")]
    AskQuestion,
    #[command(description = "отменить процесс / процесті болдырмау")]
    Cancel,
}

pub fn schema(admin_group_id: ChatId, public_group_id: ChatId) -> UpdateHandler<Box<dyn Error + Send + Sync + 'static>> {
    use dptree::case;

    // The user flow is only for private chats
    let user_flow = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_private())
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::AskQuestion].endpoint(ask_question))
                .branch(case![Command::Cancel].endpoint(cancel))
        )
        .branch(case![State::Start].endpoint(unrecognized_in_start))
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
        .endpoint(move |bot: Bot, msg: Message| admin_reply_handler(bot, msg, public_group_id));

    let callback_flow = Update::filter_callback_query()
        .endpoint(callback_handler);

    dptree::entry()
        .branch(user_flow)
        .branch(admin_flow)
        .branch(callback_flow)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // Reset state just in case
    dialogue.exit().await?;
    
    let intro = "👋 Здравствуйте! Я бот службы поддержки.\n\n\
                 Я помогу вам связаться с администраторами сайта. \
                 Если у вас есть вопросы, предложения или проблемы, я соберу ваши контактные данные \
                 и безопасно передам ваше сообщение команде.\n\n\
                 Чтобы начать и отправить запрос, пожалуйста, используйте команду /askquestion.\n\n\
                 ---\n\n\
                 👋 Сәлеметсіз бе! Мен қолдау қызметінің ботымын.\n\n\
                 Мен сізге сайт әкімшілерімен байланысуға көмектесемін. \
                 Егер сізде сұрақтар, ұсыныстар немесе мәселелер болса, мен сіздің байланыс деректеріңізді жинап, \
                 хабарламаңызды командаға қауіпсіз түрде жеткіземін.\n\n\
                 Сұранымды бастау және жіберу үшін /askquestion пәрменін пайдаланыңыз.";
                 
    bot.send_message(msg.chat.id, intro).await?;
    Ok(())
}

async fn ask_question(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Давайте начнем. Пожалуйста, введите ваше полное имя (ФИО):\n\nБастайық. Толық аты-жөніңізді енгізіңіз:")
        .await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn unrecognized_in_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Чтобы отправить запрос, пожалуйста, используйте команду /askquestion.\n\nСұраным жіберу үшін /askquestion пәрменін пайдаланыңыз.").await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Процесс отменен. Вы можете ввести /start, чтобы начать заново.\n\nПроцесс тоқтатылды. Қайта бастау үшін /start теріңіз.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Пожалуйста, введите название вашего проекта:\n\nЖобаңыздың атауын енгізіңіз:").await?;
            let mut form = FormState::default();
            form.full_name = text.into();
            dialogue.update(State::ReceiveProjectName(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Пожалуйста, отправьте текстовое сообщение.\n\nМәтіндік хабарлама жіберіңіз.").await?;
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
            bot.send_message(msg.chat.id, "Пожалуйста, введите ваши контактные данные (Email и/или номер телефона):\n\nБайланыс деректеріңізді енгізіңіз (Email және/немесе телефон нөмірі):").await?;
            form.project_name = text.into();
            dialogue.update(State::ReceiveContactInfo(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Пожалуйста, отправьте текстовое сообщение.\n\nМәтіндік хабарлама жіберіңіз.").await?;
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
            bot.send_message(msg.chat.id, "И наконец, каков ваш вопрос или сообщение?\n\nЖәне соңында, сіздің сұрағыңыз немесе хабарламаңыз қандай?").await?;
            form.contact_info = text.into();
            dialogue.update(State::ReceiveQuestion(form)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Пожалуйста, отправьте текстовое сообщение.\n\nМәтіндік хабарлама жіберіңіз.").await?;
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
            bot.send_message(msg.chat.id, "Спасибо! Мы получили ваш запрос, и наши администраторы скоро ответят вам здесь.\n\nРахмет! Біз сіздің сұранымыңызды алдық, және біздің әкімшілер жақында осында жауап береді.").await?;
            
            // Format the ticket
            let ticket = format!(
                "🚨 **Новый запрос в поддержку**\n\
                 **ID:** {}\n\
                 **Имя:** {}\n\
                 **Контакты:** {}\n\
                 ---\n\
                 **Проект:** {}\n\
                 **Вопрос:**\n{}",
                msg.chat.id, form.full_name, form.contact_info, form.project_name, question
            );
            
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("❌ Отклонить (Некорректный вопрос)", format!("reject:{}", msg.chat.id))]
            ]);
            
            // Send to Admin Group
            bot.send_message(admin_group_id, ticket).reply_markup(keyboard).await?;
            
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Пожалуйста, отправьте текстовое сообщение.\n\nМәтіндік хабарлама жіберіңіз.").await?;
        }
    }
    Ok(())
}

async fn admin_reply_handler(bot: Bot, msg: Message, public_group_id: ChatId) -> HandlerResult {
    if let Some(reply_to) = msg.reply_to_message() {
        if let Some(text) = reply_to.text() {
            // Very simple parser to extract ID
            // We look for "**ID:** 123456789"
            if let Some(id_str) = text.lines().find(|l| l.starts_with("**ID:** ")) {
                let id_str = id_str.trim_start_matches("**ID:** ");
                if let Ok(user_id) = id_str.parse::<i64>() {
                    let user_chat_id = ChatId(user_id);
                    if let Some(admin_reply_text) = msg.text() {
                        let final_message = format!("🧑‍💻 **Ответ службы поддержки / Қолдау қызметінің жауабы:**\n\n{}", admin_reply_text);
                        // Send the reply back to the user
                        if let Err(e) = bot.send_message(user_chat_id, final_message).await {
                            log::error!("Failed to send reply to user {}: {}", user_id, e);
                            bot.send_message(msg.chat.id, format!("❌ Ошибка при отправке пользователю: {}", e)).await?;
                        } else {
                            // Extract project and question for public group
                            let mut project_name = String::new();
                            let mut question_text = String::new();
                            let lines: Vec<&str> = text.lines().collect();
                            let mut in_question = false;
                            for line in &lines {
                                if line.starts_with("**Проект:** ") {
                                    project_name = line.trim_start_matches("**Проект:** ").to_string();
                                } else if line.starts_with("**Вопрос:**") {
                                    in_question = true;
                                } else if in_question {
                                    question_text.push_str(line);
                                    question_text.push('\n');
                                }
                            }
                            let question_text = question_text.trim();
                            
                            // Publish to public group
                            if !project_name.is_empty() && !question_text.is_empty() {
                                let public_msg = format!(
                                    "📢 **Новый ответ на вопрос / Жаңа сұраққа жауап**\n\
                                     **Проект / Жоба:** {}\n\n\
                                     **Вопрос / Сұрақ:**\n{}\n\n\
                                     **Ответ / Жауап:**\n{}",
                                    project_name, question_text, admin_reply_text
                                );
                                if let Err(e) = bot.send_message(public_group_id, public_msg).await {
                                    log::error!("Failed to send to public group: {}", e);
                                    bot.send_message(msg.chat.id, format!("⚠️ Ответ отправлен пользователю, но ошибка при публикации: {}", e)).await?;
                                } else {
                                    bot.send_message(msg.chat.id, "✅ Ответ успешно отправлен пользователю и опубликован в публичной группе.").await?;
                                }
                            } else {
                                bot.send_message(msg.chat.id, "✅ Ответ отправлен пользователю, но не удалось извлечь проект/вопрос для публикации.").await?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(ref data) = q.data {
        if data.starts_with("reject:") {
            let id_str = data.trim_start_matches("reject:");
            if let Ok(user_id) = id_str.parse::<i64>() {
                let user_chat_id = ChatId(user_id);
                
                // Send rejection to user
                let reject_msg = "Ваш вопрос был отклонен администратором как некорректный. Пожалуйста, сформулируйте вопрос точнее и отправьте снова с помощью /askquestion.\n\n\
                                  Әкімші сұрағыңызды қате ретінде қабылдамады. Сұрағыңызды нақтылап, /askquestion арқылы қайта жіберіңіз.";
                let _ = bot.send_message(user_chat_id, reject_msg).await;
                
                // Edit original message to show it was rejected
                if let Some(message) = q.regular_message() {
                    let old_text = message.text().unwrap_or("");
                    let new_text = format!("{}\n\n❌ **ОТКЛОНЕН (Пользователь уведомлен)**", old_text);
                    let _ = bot.edit_message_text(message.chat.id, message.id, new_text).await;
                }
            }
        }
    }
    // Answer callback query so it stops spinning
    bot.answer_callback_query(q.id).await?;
    Ok(())
}
