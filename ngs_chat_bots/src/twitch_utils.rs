use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_std::{
    io::{BufReader, WriteExt},
    net::TcpStream,
    prelude::*,
};
use anyhow::Result;

const SERVER: &str = "irc.chat.twitch.tv:6667";

static BOT_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub struct Bot {
    pub name: String,
    pub token: String,
    pub available: bool,
    pub enable: bool,
    pub chat_history: Vec<String>,
}

impl Bot {
    pub fn new(name: String, token: String) -> Self {
        Self {
            name,
            token,
            available: false,
            enable: true,
            chat_history: Vec::new(),
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        test_irc_connection(&self.name, &self.token).await
    }

    pub async fn send_message(&self, channel: &str, message: &str) -> Result<()> {
        send_message_to_channel(&self.name, &self.token, channel, message).await
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enable = enabled;
    }

    pub fn add_to_history(&mut self, message: String) {
        self.chat_history.push(message);
    }

    pub fn clear_history(&mut self) {
        self.chat_history.clear();
    }
}

pub fn create_bots(content: &str) -> Vec<Bot> {
    content
        .lines()
        .filter_map(|line| {
            if let Some((token, name_part)) = line.split_once('|') {
                let name = name_part.trim();
                let final_name = if name.is_empty() {
                    let id = BOT_COUNTER.fetch_add(1, Ordering::SeqCst);
                    format!("bot_{}", id)
                } else {
                    name.to_string()
                };
                Some(Bot::new(final_name, token.trim().to_string()))
            } else {
                let token = line.trim();
                if !token.is_empty() {
                    let id = BOT_COUNTER.fetch_add(1, Ordering::SeqCst);
                    Some(Bot::new(format!("bot_{}", id), token.to_string()))
                } else {
                    None
                }
            }
        })
        .collect()
}

async fn test_irc_connection(username: &str, oauth_token: &str) -> Result<bool> {
    let result = async_std::future::timeout(
        Duration::from_secs(10),
        async {
            let mut stream = TcpStream::connect(SERVER).await?;

            stream
                .write_all(format!("PASS oauth:{}\r\n", oauth_token).as_bytes())
                .await?;
            stream
                .write_all(format!("NICK {}\r\n", username).as_bytes())
                .await?;

            let mut buffer = vec![0u8; 2048];
            let n = stream.read(&mut buffer).await?;
            let response = String::from_utf8_lossy(&buffer[..n]);
            
            if response.contains(":tmi.twitch.tv 001") || response.contains("Welcome") {
                anyhow::Ok(true)
            } else if response.contains("Login authentication failed") 
                || response.contains("Login unsuccessful") {
                anyhow::Ok(false)
            } else {
                anyhow::Ok(false)
            }
        }
    ).await;

    match result {
        Ok(Ok(valid)) => Ok(valid),
        _ => Ok(false),
    }
}

async fn send_message_to_channel(
    nickname: &str,
    oauth: &str,
    channel: &str,
    message: &str,
) -> Result<()> {
    let stream = TcpStream::connect(SERVER).await?;
    let (reader, mut writer) = (&stream, &stream);
    
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    writer.write_all(format!("PASS oauth:{}\n", oauth).as_bytes()).await?;
    println!("Sent: PASS oauth:***");
    
    writer.write_all(format!("NICK {}\n", nickname).as_bytes()).await?;
    println!("Sent: NICK {}", nickname);
    
    writer.write_all(format!("JOIN #{}\r\n", channel).as_bytes()).await?;
    println!("Sent: JOIN #{}", channel);
    
    let timeout = std::time::Instant::now();
    let mut joined = false;
    
    while timeout.elapsed() < Duration::from_secs(5) {
        line.clear();
        if let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 { break; }
            println!("< {}", line.trim());
            
            if line.contains("366") || line.contains("End of /NAMES list") {
                joined = true;
                break;
            }
            
            if line.starts_with("PING") {
                let pong = line.replace("PING", "PONG");
                writer.write_all(pong.as_bytes()).await?;
            }
        }
    }
    
    if !joined {
        return Err(anyhow::anyhow!("Не удалось войти в канал"));
    }

    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("\n✓ Успешно вошли в канал, отправляем сообщение...\n");
    
    writer.write_all(format!("PRIVMSG #{} :{}\r\n", channel, message).as_bytes()).await?;
    println!("Sent: PRIVMSG #{} :{}", channel, message);
    
    async_std::task::sleep(Duration::from_secs(2)).await;
    
    Ok(())
}