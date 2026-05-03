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
#[command(rename_rule = "lowercase", description = "Поддерживаемые команды / Қолдау көрсетілетін пәрмендер:")]
pub enum Command {
    #[command(description = "показать приветствие / сәлемдесуді көрсету")]
    Start,
    #[command(description = "задать вопрос / сұрақ қою")]
    AskQuestion,
    #[command(description = "отменить процесс / процесті болдырмау")]
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
        .endpoint(admin_reply_handler);

    dptree::entry()
        .branch(user_flow)
        .branch(admin_flow)
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
                 **Проект:** {}\n\
                 **Контакты:** {}\n\
                 **Вопрос:** {}",
                msg.chat.id, form.full_name, form.project_name, form.contact_info, question
            );
            
            // Send to Admin Group
            bot.send_message(admin_group_id, ticket).await?;
            
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Пожалуйста, отправьте текстовое сообщение.\n\nМәтіндік хабарлама жіберіңіз.").await?;
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
                        let final_message = format!("🧑‍💻 **Ответ службы поддержки / Қолдау қызметінің жауабы:**\n\n{}", admin_reply_text);
                        // Send the reply back to the user
                        if let Err(e) = bot.send_message(user_chat_id, final_message).await {
                            log::error!("Failed to send reply to user {}: {}", user_id, e);
                            bot.send_message(msg.chat.id, format!("❌ Ошибка при отправке пользователю: {}", e)).await?;
                        } else {
                            // React or confirm success
                            bot.send_message(msg.chat.id, "✅ Ответ успешно отправлен.").await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
