# NGS-Chat-Bots

A desktop application for managing multiple Twitch chat bots with an intuitive interface.

<img width="2560" height="1380" alt="NGS-Chat-Bots Interface" src="https://github.com/user-attachments/assets/32b95e08-5ce8-475d-bb31-0f355f0ffaef" />

https://github.com/user-attachments/assets/184a84bf-4794-4cf1-9fb7-5a9d4f490d42

## LEGAL
This software is intended solely for educational and security research purposes, particularly for website security testing. The author assumes no responsibility for misuse, illegal activities, or unauthorized access carried out with this tool. By using it, you accept full responsibility for any consequences resulting from improper or unethical use.

The program functions as an automated system and does not collect, store, or share personal data. It is designed strictly for cybersecurity testing, data analysis, and ethical vulnerability research.

Any unauthorized or malicious use, including hacking, credential theft, or illegal data access, is strictly prohibited.

For legal inquiries, please contact me by email.

## Features

- **Multi-bot Management** - Control multiple Twitch bots simultaneously from a single interface
- **Manual Messaging** - Send messages on behalf of any connected bot
- **Automated Messaging** - Schedule messages to be sent at specified intervals
- **Message Templates** - Upload and manage prepared message templates for quick sending
- **Chat History** - Save and review global chat logs and individual bot message history
- **Real-time Updates** - Monitor chat activity across all connected bots in real-time

## Requirements

- Rust (latest stable version)
- Cargo package manager
- Twitch account(s) for bot(s)

## About .txt files.

1. Tokens and messages must be written as follows: 1 token or message per line
2. You can name the bot if you want. To do this, simply write |name after the token
3. The token must be without auth:

## Installation

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/ZikQ/NGS-Chat-Bots.git
cd NGS-Chat-Bots/ngs_chat_bots
```

2. Build and run the project:
```bash
cargo run
```

For optimized release build:
```bash
cargo build --release
./target/release/ngs_chat_bots
```

### Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/ZikQ/NGS-Chat-Bots/releases/tag/release).

## Usage

1. Launch the application
2. Add your Twitch bot credentials
3. Connect to desired Twitch channels
4. Start managing your bots through the interface

## Author

[ZikQ](https://github.com/ZikQ)

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/ZikQ/NGS-Chat-Bots/issues) on GitHub.

---

**Give it a ⭐ star to show your support!**
