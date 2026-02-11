use iced::{Alignment, Background, Border, Color, Element, Length, Subscription, Task, 
    widget::{button, checkbox, column, container, row, scrollable, text, text_editor, text_input}};
use rfd::FileDialog;
use std::time::{Duration, Instant};
use rand::Rng;

use crate::twitch_utils::{Bot, create_bots};

#[derive(Debug, Clone)]
pub enum Message {
    None, 

    LoadedMessages(String),
    LoadedConfig(String),
    BotChecked(usize, bool),
    ToggleBotEnabled(usize, bool),
    
    LoadMessagesPress,
    LoadConfigPress,
    CheckBotsPress,
    
    MessageUpdated(String),
    ChannelNameUpdated(String),
    
    MessageSent(usize, Result<(), String>),
    
    SendMessage(usize),
    SendMessageAllBots,
    SendMessageRandomBot,
    
    ToggleRandomMessages(bool),
    MinIntervalUpdated(String),
    MaxIntervalUpdated(String),
    Tick(Instant),
    SendRandomMessage,
    SendRandomMessageNow,
    
    ToggleAllBotsMode(bool),
    ToggleSimultaneousMode(bool),
    MinBotDelayUpdated(String),
    MaxBotDelayUpdated(String),
    
    ToggleMultipleBotsMode(bool),
    MultipleBotsCountUpdated(String),
    
    ToggleClearAfterSend(bool),
    
    MessagesEditorAction(text_editor::Action),
    
    ToggleBotChatView(usize),
    CloseBotChatView,
    BotMessageUpdated(String),
    SendBotMessage(usize),
    ClearBotHistory(usize),
    ClearGlobalHistory,
    ClearAllHistory,
    
    SearchQueryUpdated(String),
    
    MessageClicked(usize),
}

pub struct App {
    message: String,
    bots: Vec<Bot>,
    chat_history: Vec<String>,
    channel: String,
    messages: Vec<String>,
    
    random_messages_enabled: bool,
    min_interval: u64,
    max_interval: u64,
    next_message_time: Option<Instant>,
    last_message_time: Option<Instant>,
    
    all_bots_mode: bool,
    simultaneous_mode: bool,
    min_bot_delay: u64,
    max_bot_delay: u64,
    
    multiple_bots_mode: bool,
    multiple_bots_count: usize,
    
    clear_after_send: bool,
    
    messages_editor: text_editor::Content,
    
    viewing_bot_chat: Option<usize>,
    bot_message_input: String,
    
    search_query: String,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            bots: Vec::new(),
            chat_history: Vec::new(),
            message: String::new(),
            channel: String::new(),
            messages: Vec::new(),
            random_messages_enabled: false,
            min_interval: 30,
            max_interval: 120,
            next_message_time: None,
            last_message_time: None,
            all_bots_mode: false,
            simultaneous_mode: true,
            min_bot_delay: 1,
            max_bot_delay: 3,
            multiple_bots_mode: false,
            multiple_bots_count: 3,
            clear_after_send: false,
            messages_editor: text_editor::Content::new(),
            viewing_bot_chat: None,
            bot_message_input: String::new(),
            search_query: String::new(),
        };

        return (app, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MessageUpdated(message) => {
                self.message = message;
                Task::none()
            },
            Message::LoadConfigPress => {
                let file = FileDialog::new()
                    .add_filter("Text Document", &["txt"]);
    
                if let Some(path) = file.pick_file() {
                    Task::perform(
                        async move {
                            match std::fs::read_to_string(path) {
                                Ok(content) => Message::LoadedConfig(content),
                                Err(_) => Message::None,
                            }
                        },
                        |msg| msg
                    )
                } else {
                    Task::none()
                }
            },
            Message::LoadedConfig(content) => {
                self.bots = create_bots(&content);
                Task::none()
            },
            Message::CheckBotsPress => {
                let mut tasks: Vec<Task<Message>> = Vec::new();

                for (index, bot) in self.bots.iter().enumerate() {
                    let bot_clone = bot.clone();
        
                    tasks.push(
                        Task::perform(
                            async move {
                                bot_clone.test_connection().await
                            },
                            move |result| {
                                match result {
                                    Ok(is_valid) => Message::BotChecked(index, is_valid),
                                    Err(_) => Message::BotChecked(index, false),
                                }
                            }
                        )
                    );
                }

                Task::batch(tasks)
            },
            Message::BotChecked(index, flag) => {
                if let Some(bot) = self.bots.get_mut(index) {
                    bot.set_available(flag);
                }
                Task::none()
            },
            Message::ToggleBotEnabled(index, enabled) => {
                if let Some(bot) = self.bots.get_mut(index) {
                    bot.set_enabled(enabled);
                }
                Task::none()
            },
            Message::SendMessage(index) => {
                if let Some(bot) = self.bots.get(index) {
                    if !bot.available || !bot.enable {
                        return Task::none();
                    }

                    let bot_clone = bot.clone();
                    let channel = self.channel.clone();
                    let message = self.message.clone();

                    self.chat_history.push(format!("[{}] {}", bot.name, message));
                    
                    if let Some(bot) = self.bots.get_mut(index) {
                        bot.add_to_history(format!("[{}] {}", bot.name, message));
                    }
                    
                    if self.clear_after_send {
                        self.message.clear();
                    }

                    Task::perform(
                        async move {
                            bot_clone.send_message(&channel, &message).await
                        },
                        move |result| {
                            match result {
                                Ok(_) => Message::MessageSent(index, Ok(())),
                                Err(e) => Message::MessageSent(index, Err(e.to_string())),
                            }
                        }
                    )
                } else {
                    Task::none()
                }
            },

            Message::SendMessageRandomBot => {
                if self.message.is_empty() || self.channel.is_empty() {
                    return Task::none();
                }

                let available_bots: Vec<usize> = self.bots
                    .iter()
                    .enumerate()
                    .filter(|(_, bot)| bot.available && bot.enable)
                    .map(|(idx, _)| idx)
                    .collect();

                if available_bots.is_empty() {
                    return Task::none();
                }

                let mut rng = rand::rng();
                let bot_index = available_bots[rng.random_range(0..available_bots.len())];

                if let Some(bot) = self.bots.get(bot_index) {
                    let bot_clone = bot.clone();
                    let channel = self.channel.clone();
                    let message = self.message.clone();
                    self.chat_history.push(format!("[🎲 {}] {}", bot.name, message));
                    
                    if let Some(bot_mut) = self.bots.get_mut(bot_index) {
                        bot_mut.add_to_history(format!("[🎲 {}] {}", bot_mut.name, message));
                    }
                    
                    if self.clear_after_send {
                        self.message.clear();
                    }

                    Task::perform(
                        async move {
                            bot_clone.send_message(&channel, &message).await
                        },
                        move |result| {
                            match result {
                                Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                            }
                        }
                    )
                } else {
                    Task::none()
                }
            },
            Message::SendMessageAllBots => {
                if self.message.is_empty() || self.channel.is_empty() {
                    return Task::none();
                }

                let available_bots: Vec<usize> = self.bots
                    .iter()
                    .enumerate()
                    .filter(|(_, bot)| bot.available && bot.enable)
                    .map(|(idx, _)| idx)
                    .collect();

                if available_bots.is_empty() {
                    return Task::none();
                }

                let message = self.message.clone();
                let mut tasks = Vec::new();
                let mut rng = rand::rng();

                for (delay_index, &bot_index) in available_bots.iter().enumerate() {
                    if let Some(bot) = self.bots.get(bot_index) {
                        let bot_clone = bot.clone();
                        let channel = self.channel.clone();
                        let msg_clone = message.clone();
                        
                        self.chat_history.push(format!("[{}] {}", bot.name, message));
                        
                        if let Some(bot_mut) = self.bots.get_mut(bot_index) {
                            bot_mut.add_to_history(format!("[{}] {}", bot_mut.name, message));
                        }
                        
                        if self.simultaneous_mode {
                            tasks.push(Task::perform(
                                async move {
                                    bot_clone.send_message(&channel, &msg_clone).await
                                },
                                move |result| {
                                    match result {
                                        Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                        Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                    }
                                }
                            ));
                        } else {
                            let delay = rng.random_range(self.min_bot_delay..=self.max_bot_delay) * delay_index as u64;
                            tasks.push(Task::perform(
                                async move {
                                    if delay > 0 {
                                        async_std::task::sleep(Duration::from_secs(delay)).await;
                                    }
                                    bot_clone.send_message(&channel, &msg_clone).await
                                },
                                move |result| {
                                    match result {
                                        Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                        Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                    }
                                }
                            ));
                        }
                    }
                }
                
                if self.clear_after_send {
                    self.message.clear();
                }

                Task::batch(tasks)
            },
            Message::MessageSent(index, result) => {
                if let Err(error) = result {
                    let error_msg = format!("❌ Error: {}", error);
                    self.chat_history.push(error_msg.clone());
                    
                    if let Some(bot) = self.bots.get_mut(index) {
                        bot.add_to_history(error_msg);
                    }
                }
                Task::none()
            },
            Message::None => Task::none(),
            Message::ChannelNameUpdated(name) => {
                self.channel = name;
                Task::none()
            },
            Message::LoadedMessages(messages) => {
                self.messages = messages
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                self.messages_editor = text_editor::Content::with_text(&messages);

                Task::none()
            },
            Message::LoadMessagesPress => {
                let file = FileDialog::new()
                    .add_filter("Text Document", &["txt"]);
    
                if let Some(path) = file.pick_file() {
                    Task::perform(
                        async move {
                            match std::fs::read_to_string(path) {
                                Ok(content) => Message::LoadedMessages(content),
                                Err(_) => Message::None,
                            }
                        },
                        |msg| msg
                    )
                } else {
                    Task::none()
                }
            },

            Message::ToggleRandomMessages(enabled) => {
                self.random_messages_enabled = enabled;
                if enabled {
                    self.schedule_next_message();
                } else {
                    self.next_message_time = None;
                }
                Task::none()
            },
            Message::MinIntervalUpdated(value) => {
                if let Ok(val) = value.parse::<u64>() {
                    self.min_interval = val;
                }
                Task::none()
            },
            Message::MaxIntervalUpdated(value) => {
                if let Ok(val) = value.parse::<u64>() {
                    self.max_interval = val;
                }
                Task::none()
            },
            Message::Tick(now) => {
                if self.random_messages_enabled {
                    if let Some(next_time) = self.next_message_time {
                        if now >= next_time {
                            return Task::done(Message::SendRandomMessage);
                        }
                    }
                }
                Task::none()
            },
            Message::SendRandomMessageNow => {
                self.next_message_time = None;
                Task::done(Message::SendRandomMessage)
            },
            Message::SendRandomMessage => {
                if self.messages.is_empty() || self.channel.is_empty() {
                    return Task::none();
                }

                let available_bots: Vec<usize> = self.bots
                    .iter()
                    .enumerate()
                    .filter(|(_, bot)| bot.available && bot.enable)
                    .map(|(idx, _)| idx)
                    .collect();

                if available_bots.is_empty() {
                    return Task::none();
                }

                let mut rng = rand::rng();

                if self.multiple_bots_mode {
                    let mut tasks = Vec::new();
                    let bots_to_use = self.multiple_bots_count.min(available_bots.len());
                    
                    let mut shuffled_bots = available_bots.clone();
                    use rand::seq::SliceRandom;
                    shuffled_bots.shuffle(&mut rng);
                    
                    for i in 0..bots_to_use {
                        let bot_index = shuffled_bots[i];
                        
                        if let Some(bot) = self.bots.get(bot_index) {
                            let message = if self.messages.len() > i {
                                let mut shuffled_messages = self.messages.clone();
                                shuffled_messages.shuffle(&mut rng);
                                shuffled_messages[i % shuffled_messages.len()].clone()
                            } else {
                                self.messages[rng.random_range(0..self.messages.len())].clone()
                            };
                            
                            let bot_clone = bot.clone();
                            let channel = self.channel.clone();
                            let msg_clone = message.clone();
                            
                            let history_msg = format!("[🎲 {}] {}", bot.name, message);
                            self.chat_history.push(history_msg.clone());
                            
                            if let Some(bot_mut) = self.bots.get_mut(bot_index) {
                                bot_mut.add_to_history(history_msg);
                            }
                            
                            if self.simultaneous_mode {
                                tasks.push(Task::perform(
                                    async move {
                                        bot_clone.send_message(&channel, &msg_clone).await
                                    },
                                    move |result| {
                                        match result {
                                            Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                            Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                        }
                                    }
                                ));
                            } else {
                                let delay = rng.random_range(self.min_bot_delay..=self.max_bot_delay) * i as u64;
                                tasks.push(Task::perform(
                                    async move {
                                        if delay > 0 {
                                            async_std::task::sleep(Duration::from_secs(delay)).await;
                                        }
                                        bot_clone.send_message(&channel, &msg_clone).await
                                    },
                                    move |result| {
                                        match result {
                                            Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                            Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                        }
                                    }
                                ));
                            }
                        }
                    }
                    
                    self.last_message_time = Some(Instant::now());
                    
                    if self.random_messages_enabled {
                        self.schedule_next_message();
                    }
                    
                    return Task::batch(tasks);
                }
                else if self.all_bots_mode {
                    let message = self.messages[rng.random_range(0..self.messages.len())].clone();
                    
                    let mut tasks = Vec::new();
                    
                    for (delay_index, &bot_index) in available_bots.iter().enumerate() {
                        if let Some(bot) = self.bots.get(bot_index) {
                            let bot_clone = bot.clone();
                            let channel = self.channel.clone();
                            let msg_clone = message.clone();
                            
                            let history_msg = format!("[🎲 {}] {}", bot.name, message);
                            self.chat_history.push(history_msg.clone());
                            
                            if let Some(bot_mut) = self.bots.get_mut(bot_index) {
                                bot_mut.add_to_history(history_msg);
                            }
                            
                            if self.simultaneous_mode {
                                tasks.push(Task::perform(
                                    async move {
                                        bot_clone.send_message(&channel, &msg_clone).await
                                    },
                                    move |result| {
                                        match result {
                                            Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                            Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                        }
                                    }
                                ));
                            } else {
                                let delay = rng.random_range(self.min_bot_delay..=self.max_bot_delay) * delay_index as u64;
                                tasks.push(Task::perform(
                                    async move {
                                        if delay > 0 {
                                            async_std::task::sleep(Duration::from_secs(delay)).await;
                                        }
                                        bot_clone.send_message(&channel, &msg_clone).await
                                    },
                                    move |result| {
                                        match result {
                                            Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                            Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                        }
                                    }
                                ));
                            }
                        }
                    }
                    
                    self.last_message_time = Some(Instant::now());
                    
                    if self.random_messages_enabled {
                        self.schedule_next_message();
                    }
                    
                    return Task::batch(tasks);
                } 
                else {
                    let message = self.messages[rng.random_range(0..self.messages.len())].clone();
                    let bot_index = available_bots[rng.random_range(0..available_bots.len())];
                    
                    if let Some(bot) = self.bots.get(bot_index) {
                        let bot_clone = bot.clone();
                        let channel = self.channel.clone();
                        let msg_clone = message.clone();

                        let history_msg = format!("[🎲 {}] {}", bot.name, message);
                        self.chat_history.push(history_msg.clone());
                        
                        if let Some(bot_mut) = self.bots.get_mut(bot_index) {
                            bot_mut.add_to_history(history_msg);
                        }
                        
                        self.last_message_time = Some(Instant::now());
                        
                        if self.random_messages_enabled {
                            self.schedule_next_message();
                        }

                        return Task::perform(
                            async move {
                                bot_clone.send_message(&channel, &msg_clone).await
                            },
                            move |result| {
                                match result {
                                    Ok(_) => Message::MessageSent(bot_index, Ok(())),
                                    Err(e) => Message::MessageSent(bot_index, Err(e.to_string())),
                                }
                            }
                        );
                    }
                }
                
                Task::none()
            },
            Message::ToggleAllBotsMode(enabled) => {
                self.all_bots_mode = enabled;
                if enabled && self.multiple_bots_mode {
                    self.multiple_bots_mode = false;
                }
                Task::none()
            },
            Message::ToggleSimultaneousMode(enabled) => {
                self.simultaneous_mode = enabled;
                Task::none()
            },
            Message::MinBotDelayUpdated(value) => {
                if let Ok(val) = value.parse::<u64>() {
                    self.min_bot_delay = val;
                }
                Task::none()
            },
            Message::MaxBotDelayUpdated(value) => {
                if let Ok(val) = value.parse::<u64>() {
                    self.max_bot_delay = val;
                }
                Task::none()
            },
            Message::ToggleMultipleBotsMode(enabled) => {
                self.multiple_bots_mode = enabled;
                if enabled && self.all_bots_mode {
                    self.all_bots_mode = false;
                }
                Task::none()
            },
            Message::MultipleBotsCountUpdated(value) => {
                if let Ok(val) = value.parse::<usize>() {
                    if val > 0 {
                        self.multiple_bots_count = val;
                    }
                }
                Task::none()
            },
            Message::ToggleClearAfterSend(enabled) => {
                self.clear_after_send = enabled;
                Task::none()
            },
            Message::MessagesEditorAction(action) => {
                self.messages_editor.perform(action);
                
                let editor_text = self.messages_editor.text();
                self.messages = editor_text
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect();
                
                Task::none()
            },
            Message::ToggleBotChatView(index) => {
                self.viewing_bot_chat = Some(index);
                self.bot_message_input.clear();
                Task::none()
            },
            Message::CloseBotChatView => {
                self.viewing_bot_chat = None;
                self.bot_message_input.clear();
                Task::none()
            },
            Message::BotMessageUpdated(msg) => {
                self.bot_message_input = msg;
                Task::none()
            },
            Message::SendBotMessage(index) => {
                if self.bot_message_input.is_empty() || self.channel.is_empty() {
                    return Task::none();
                }
                
                if let Some(bot) = self.bots.get(index) {
                    if !bot.available || !bot.enable {
                        return Task::none();
                    }

                    let bot_clone = bot.clone();
                    let channel = self.channel.clone();
                    let message = self.bot_message_input.clone();

                    let history_msg = format!("[{}] {}", bot.name, message);
                    self.chat_history.push(history_msg.clone());
                    
                    if let Some(bot_mut) = self.bots.get_mut(index) {
                        bot_mut.add_to_history(history_msg);
                    }
                    
                    self.bot_message_input.clear();

                    Task::perform(
                        async move {
                            bot_clone.send_message(&channel, &message).await
                        },
                        move |result| {
                            match result {
                                Ok(_) => Message::MessageSent(index, Ok(())),
                                Err(e) => Message::MessageSent(index, Err(e.to_string())),
                            }
                        }
                    )
                } else {
                    Task::none()
                }
            },
            Message::ClearBotHistory(index) => {
                if let Some(bot) = self.bots.get_mut(index) {
                    bot.clear_history();
                }
                Task::none()
            },
            Message::ClearGlobalHistory => {
                self.chat_history.clear();
                Task::none()
            },

            Message::ClearAllHistory => {
                self.chat_history.clear();
                for bot in &mut self.bots {
                    bot.clear_history();
                }
                Task::none()
            },

            Message::SearchQueryUpdated(query) => {
                self.search_query = query;
                Task::none()
            },

            Message::MessageClicked(message_index) => {
                if let Some(msg) = self.chat_history.get(message_index) {

                    if let Some(bot_name) = self.extract_bot_name_from_message(msg) {
                        if let Some(bot_index) = self.bots.iter().position(|b| b.name == bot_name) {
                            self.viewing_bot_chat = Some(bot_index);
                            self.bot_message_input.clear();
                        }
                    }
                }
                Task::none()
            },
        }
    }

    fn extract_bot_name_from_message(&self, message: &str) -> Option<String> {
        if let Some(start) = message.find('[') {
            if let Some(end) = message.find(']') {
                let bot_part = &message[start+1..end];
                let bot_name = bot_part.trim_start_matches("🎲 ").trim();
                return Some(bot_name.to_string());
            }
        }
        None
    }

    fn get_filtered_bots(&self) -> Vec<(usize, &Bot)> {
        if self.search_query.is_empty() {
            self.bots.iter().enumerate().collect()
        } else {
            self.bots
                .iter()
                .enumerate()
                .filter(|(_, bot)| {
                    bot.name.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }

    fn schedule_next_message(&mut self) {
        let mut rng = rand::rng();
        let interval = rng.random_range(self.min_interval..=self.max_interval);
        self.next_message_time = Some(Instant::now() + Duration::from_secs(interval));
    }

    pub fn view(&self) -> Element<Message> {
        if let Some(bot_index) = self.viewing_bot_chat {
            return self.view_bot_chat(bot_index);
        }

        let header = container(
            text("NGS Chat Bot Utils")
                .size(24)
        )
        .padding(20)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
            ..Default::default()
        });

        let controls = container(
            column![
                row![
                    button(text("📁 Load Tokens"))
                        .on_press(Message::LoadConfigPress)
                        .padding(10),
                    button(text("⚙️ Check Bots"))
                        .on_press(Message::CheckBotsPress)
                        .padding(10),
                    button(text("💌 Load Messages"))
                        .on_press(Message::LoadMessagesPress)
                        .padding(10),
                    text(format!("Loaded: {}", self.messages.len()))
                        .size(14),
                    button(text("🗑️ Clear Global Chat"))
                        .on_press(Message::ClearGlobalHistory)
                        .padding(10),
                    button(text("🗑️ Clear All"))
                        .on_press(Message::ClearAllHistory)
                        .padding(10),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text_input("Channel Name", &self.channel)
                        .on_input(Message::ChannelNameUpdated)
                        .padding(10),
                ]
                .spacing(10),

                container(
                    column![
                        row![
                            checkbox(self.random_messages_enabled)
                                .on_toggle(Message::ToggleRandomMessages),
                            text("Random Messages")
                                .size(14),
                            text(format!(
                                "Status: {}",
                                if self.random_messages_enabled { 
                                    if let Some(next) = self.next_message_time {
                                        let remaining = next.saturating_duration_since(Instant::now()).as_secs();
                                        format!("✅ Active (next in {} sec)", remaining)
                                    } else {
                                        "⏳ Waiting".to_string()
                                    }
                                } else { 
                                    "❌ Disabled".to_string() 
                                }
                            ))
                            .size(14),
                            button(text("▶️ Send Now"))
                                .on_press_maybe(
                                    if !self.messages.is_empty() 
                                        && !self.channel.is_empty() 
                                        && self.bots.iter().any(|b| b.available && b.enable) {
                                        Some(Message::SendRandomMessageNow)
                                    } else {
                                        None
                                    }
                                )
                                .padding(8),
                        ]
                        .spacing(15)
                        .align_y(Alignment::Center),
                        row![
                            text("Interval (sec):").size(14),
                            text("From:").size(14),
                            text_input("", &self.min_interval.to_string())
                                .on_input(Message::MinIntervalUpdated)
                                .padding(5)
                                .width(Length::Fixed(80.0)),
                            text("To:").size(14),
                            text_input("", &self.max_interval.to_string())
                                .on_input(Message::MaxIntervalUpdated)
                                .padding(5)
                                .width(Length::Fixed(80.0)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        row![
                            checkbox(self.all_bots_mode)
                                .on_toggle(Message::ToggleAllBotsMode),
                            text("Send with all bots (same message)")
                                .size(14),
                        ]
                        .spacing(15)
                        .align_y(Alignment::Center),
                        row![
                            checkbox(self.multiple_bots_mode)
                                .on_toggle(Message::ToggleMultipleBotsMode),
                            text("Multiple bots (different messages)")
                                .size(14),
                        ]
                        .spacing(15)
                        .align_y(Alignment::Center),
                        if self.multiple_bots_mode {
                            column![
                                row![
                                    text("Number of bots:").size(14),
                                    text_input("", &self.multiple_bots_count.to_string())
                                        .on_input(Message::MultipleBotsCountUpdated)
                                        .padding(5)
                                        .width(Length::Fixed(80.0)),
                                ]
                                .spacing(10)
                                .align_y(Alignment::Center),
                                row![
                                    checkbox(self.simultaneous_mode)
                                        .on_toggle(Message::ToggleSimultaneousMode),
                                    text("Simultaneously")
                                        .size(14),
                                ]
                                .spacing(15)
                                .align_y(Alignment::Center),
                                if !self.simultaneous_mode {
                                    row![
                                        text("Delay between bots (sec):").size(14),
                                        text("From:").size(14),
                                        text_input("", &self.min_bot_delay.to_string())
                                            .on_input(Message::MinBotDelayUpdated)
                                            .padding(5)
                                            .width(Length::Fixed(80.0)),
                                        text("To:").size(14),
                                        text_input("", &self.max_bot_delay.to_string())
                                            .on_input(Message::MaxBotDelayUpdated)
                                            .padding(5)
                                            .width(Length::Fixed(80.0)),
                                    ]
                                    .spacing(10)
                                    .align_y(Alignment::Center)
                                } else {
                                    row![].into()
                                }
                            ]
                            .spacing(8)
                        } else if self.all_bots_mode {
                            column![
                                row![
                                    checkbox(self.simultaneous_mode)
                                        .on_toggle(Message::ToggleSimultaneousMode),
                                    text("Simultaneously")
                                        .size(14),
                                ]
                                .spacing(15)
                                .align_y(Alignment::Center),
                                if !self.simultaneous_mode {
                                    row![
                                        text("Delay between bots (sec):").size(14),
                                        text("From:").size(14),
                                        text_input("", &self.min_bot_delay.to_string())
                                            .on_input(Message::MinBotDelayUpdated)
                                            .padding(5)
                                            .width(Length::Fixed(80.0)),
                                        text("To:").size(14),
                                        text_input("", &self.max_bot_delay.to_string())
                                            .on_input(Message::MaxBotDelayUpdated)
                                            .padding(5)
                                            .width(Length::Fixed(80.0)),
                                    ]
                                    .spacing(10)
                                    .align_y(Alignment::Center)
                                } else {
                                    row![].into()
                                }
                            ]
                            .spacing(8)
                        } else {
                            column![].into()
                        }
                    ]
                    .spacing(10)
                )
                .padding(10)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.22))),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            ]
            .spacing(10)
        )
        .padding(15)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
            ..Default::default()
        });

        let bot_list = {
            let mut bot_column = column![].spacing(8);
            
            bot_column = bot_column.push(
                text_input("🔍 Search bot by name...", &self.search_query)
                    .on_input(Message::SearchQueryUpdated)
                    .padding(10)
                    .width(Length::Fill)
            );
            
            let filtered_bots = self.get_filtered_bots();
            
            if filtered_bots.is_empty() {
                bot_column = bot_column.push(
                    container(
                        text(
                            if self.bots.is_empty() {
                                "No bots loaded"
                            } else {
                                "No bots found"
                            }
                        ).size(14)
                    )
                    .padding(20)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .style(|_| container::Style {
                        text_color: Some(Color::from_rgb(0.5, 0.5, 0.5)),
                        ..Default::default()
                    })
                );
            } else {
                for &(index, bot) in &filtered_bots {
                    bot_column = bot_column.push(self.get_bot_panel(bot.clone(), index));
                }
            }

            container(
                column![
                    container(
                        row![
                            text("Bots").size(16),
                            text(format!("({}/{})", filtered_bots.len(), self.bots.len()))
                                .size(14)
                                .style(|_| text::Style {
                                    color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                                })
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.22))),
                        ..Default::default()
                    }),
                    scrollable(bot_column)
                        .height(Length::Fill)
                ]
            )
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.22, 0.22, 0.26))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        };

        let messages_editor_widget = container(
            column![
                container(text("Message Editor").size(16))
                    .padding(10)
                    .width(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.22))),
                        ..Default::default()
                    }),
                scrollable(
                    text_editor(&self.messages_editor)
                        .on_action(Message::MessagesEditorAction)
                        .height(Length::Fill)
                )
                .height(Length::Fill)
            ]
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.22, 0.22, 0.26))),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let chat_area = {
            let mut message_column = column![].spacing(5);
            
            if self.chat_history.is_empty() {
                message_column = message_column.push(
                    container(text("No messages yet").size(14))
                        .padding(20)
                        .width(Length::Fill)
                        .center_x(Length::Fill)
                        .style(|_| container::Style {
                            text_color: Some(Color::from_rgb(0.5, 0.5, 0.5)),
                            ..Default::default()
                        })
                );
            } else {
                for (idx, msg) in self.chat_history.iter().enumerate() {
                    let message_button = button(
                        container(text(msg).size(14))
                            .padding(8)
                            .width(Length::Fill)
                            .style(|_| container::Style {
                                background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.3))),
                                border: Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                    )
                    .on_press(Message::MessageClicked(idx))
                    .style(|_, _| button::Style {
                        background: None,
                        border: Border::default(),
                        ..Default::default()
                    })
                    .padding(0);
                    
                    message_column = message_column.push(message_button);
                }
            }

            container(
                column![
                    container(
                        row![
                            text("Global Chat").size(16),
                            text("(Click on a message to open bot chat)")
                                .size(11)
                                .style(|_| text::Style {
                                    color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                                })
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.22))),
                        ..Default::default()
                    }),
                    container(
                        scrollable(message_column)
                            .height(Length::Fill)
                    )
                    .padding(10)
                    .height(Length::FillPortion(5)),
                    container(
                        column![
                            row![
                                text_input("Enter message...", &self.message)
                                    .on_input(Message::MessageUpdated)
                                    .on_submit_maybe(
                                        if !self.message.is_empty() && self.bots.iter().any(|b| b.available && b.enable) {
                                            self.bots.iter()
                                                .position(|b| b.available && b.enable)
                                                .map(Message::SendMessage)
                                        } else {
                                            None
                                        }
                                    )
                                    .padding(10),
                                button(text("🎲 Random"))
                                    .on_press_maybe(
                                        if !self.message.is_empty() 
                                            && !self.channel.is_empty() 
                                            && self.bots.iter().any(|b| b.available && b.enable) {
                                            Some(Message::SendMessageRandomBot)
                                        } else {
                                            None
                                        }
                                    )
                                    .padding(10),
                                button(text("👥 All Bots"))
                                    .on_press_maybe(
                                        if !self.message.is_empty() 
                                            && !self.channel.is_empty() 
                                            && self.bots.iter().any(|b| b.available && b.enable) {
                                            Some(Message::SendMessageAllBots)
                                        } else {
                                            None
                                        }
                                    )
                                    .padding(10),
                            ]
                            .spacing(10),
                            row![
                                checkbox(self.clear_after_send)
                                    .on_toggle(Message::ToggleClearAfterSend),
                                text("Clear after send")
                                    .size(14),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                        ]
                        .spacing(8)
                    )
                    .padding(10)
                    .width(Length::Fill)
                ]
            )
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.22, 0.22, 0.26))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        };

        let body = container(
            column![
                controls,
                container(
                    row![
                        bot_list,
                        messages_editor_widget,
                        chat_area,
                    ]
                    .spacing(15)
                )
                .padding(15)
                .height(Length::Fill)
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            column![header, body]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
            ..Default::default()
        })
        .into()
    }

    fn view_bot_chat(&self, bot_index: usize) -> Element<Message> {
        let bot = &self.bots[bot_index];
        
        let header = container(
            row![
                button(text("← Back"))
                    .on_press(Message::CloseBotChatView)
                    .padding(10),
                text(format!("Bot Chat: {}", bot.name))
                    .size(24),
                button(text("🗑️ Clear History"))
                    .on_press(Message::ClearBotHistory(bot_index))
                    .padding(10),
            ]
            .spacing(20)
            .align_y(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
            ..Default::default()
        });

        let mut message_column = column![].spacing(5);
        
        if bot.chat_history.is_empty() {
            message_column = message_column.push(
                container(text("History is empty").size(14))
                    .padding(20)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .style(|_| container::Style {
                        text_color: Some(Color::from_rgb(0.5, 0.5, 0.5)),
                        ..Default::default()
                    })
            );
        } else {
            for msg in &bot.chat_history {
                message_column = message_column.push(
                    container(text(msg).size(14))
                        .padding(8)
                        .width(Length::Fill)
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.3))),
                            border: Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                );
            }
        }

        let chat_area = container(
            column![
                container(
                    scrollable(message_column)
                        .height(Length::Fill)
                )
                .padding(10)
                .height(Length::Fill),
                container(
                    row![
                        text_input("Enter message...", &self.bot_message_input)
                            .on_input(Message::BotMessageUpdated)
                            .on_submit_maybe(
                                if !self.bot_message_input.is_empty() && bot.available && bot.enable {
                                    Some(Message::SendBotMessage(bot_index))
                                } else {
                                    None
                                }
                            )
                            .padding(10),
                        button(text("📤 Send"))
                            .on_press_maybe(
                                if !self.bot_message_input.is_empty() 
                                    && !self.channel.is_empty() 
                                    && bot.available && bot.enable {
                                    Some(Message::SendBotMessage(bot_index))
                                } else {
                                    None
                                }
                            )
                            .padding(10),
                    ]
                    .spacing(10)
                )
                .padding(10)
                .width(Length::Fill)
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.22, 0.22, 0.26))),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        container(
            column![
                header,
                container(chat_area)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
            ..Default::default()
        })
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.random_messages_enabled {
            iced::time::every(Duration::from_millis(1000))
                .map(Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn get_bot_panel(&self, bot: Bot, index: usize) -> Element<Message> {
        let available = bot.available;
        let enabled = bot.enable;
        
        let status_color = if !enabled {
            Color::from_rgb(0.5, 0.5, 0.5)
        } else if available {
            Color::from_rgb(0.2, 0.8, 0.3)
        } else {
            Color::from_rgb(0.8, 0.2, 0.2)
        };
        
        let status_indicator = text("●")
            .style(move |_| text::Style {
                color: Some(status_color)
            });

        let status_text = text(
            if !enabled {
                "Disabled"
            } else if available {
                "Available"
            } else {
                "Unavailable"
            }
        )
        .size(12)
        .style(move |_| text::Style {
            color: Some(status_color)
        });

        let content = container(
            row![
                column![
                    text(bot.name.clone()).size(14),
                    row![
                        status_indicator,
                        status_text
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                    text(format!("Messages: {}", bot.chat_history.len()))
                        .size(11)
                        .style(|_| text::Style {
                            color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                        })
                ]
                .spacing(4),
                row![
                    checkbox(enabled)
                        .on_toggle(move |checked| Message::ToggleBotEnabled(index, checked)),
                    button(text("💬"))
                        .on_press(Message::ToggleBotChatView(index))
                        .padding(5),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding(10)
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.28, 0.28, 0.32))),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let btn = button(content)
            .style(|_, _| button::Style {
                background: None,
                border: Border::default(),
                ..Default::default()
            })
            .padding(0);

        if available && enabled && !self.message.is_empty() {
            btn.on_press(Message::SendMessage(index)).into()
        } else {
            btn.into()
        }
    }
}