# Deploying Your Telegram Bot to a Google Cloud VM

Deploying a Rust Telegram bot to a Google Cloud Platform (GCP) Virtual Machine ensures your bot runs 24/7. Here is the step-by-step guide to get it running robustly as a background service.

## 0. Telegram Groups Configuration (Two-Group Architecture)

Before deploying the bot, you need to set up the two groups required for the bot's workflow:

1. **Admin Review Group (Private):** Create a new private group. Add your trusted administrators to this group. The bot will send all incoming user questions here for review.
2. **Public Members Group (Public/Private):** This is where all regular users gather. Approved Q&As will be published here by the bot. 
   - **Moderation:** Since users are allowed to chat here, it is highly recommended to add a popular moderation bot (e.g., @MissRose_bot, @GroupHelpBot) to this group. You can configure the moderation bot to enforce rate limits (e.g., max messages per minute) and enable language filters to prevent spam and toxicity.
3. Add your newly created Rust Bot to *both* groups and ensure it has Admin permissions in both (so it can read messages and pin/delete if necessary).
4. Obtain the Chat IDs for both groups (often starting with a minus sign, e.g., `-100123456789`). You can use bots like @RawDataBot or @getidsbot to find these IDs.

## 1. Create a Google Cloud VM

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Navigate to **Compute Engine > VM instances**.
3. Click **Create Instance**.
4. **Machine Configuration:** For this bot, an `e2-micro` (in the `Shared-core` section) is more than enough and often falls under the Always Free tier.
5. **Boot Disk:** Change the OS to **Debian** or **Ubuntu** (Ubuntu 22.04 LTS or Debian 12 are great choices).
6. **Firewall:** You don't need to open any HTTP/HTTPS ports because the bot uses long-polling (it reaches *out* to Telegram).
7. Click **Create** and wait for the VM to start.

## 2. Connect to the VM

Click the **SSH** button next to your VM instance in the Google Cloud console. This will open a terminal in your browser.

## 3. Install Dependencies on the VM

Run the following commands in your VM terminal to install Rust and required compilation tools:

```bash
# Update package list and install required libraries
sudo apt update
sudo apt install -y build-essential git pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure your current shell to use Rust
source $HOME/.cargo/env
```

## 4. Transfer Your Code to the VM

The easiest way to get your code onto the VM is using Git. 
1. Push your local `telegram_bot` folder to a private GitHub repository.
2. In the VM, clone it:
   ```bash
   git clone https://github.com/yourusername/telegram_bot.git
   cd telegram_bot
   ```

*(Alternatively, you can click the "Upload File" button in the top right of the Google Cloud SSH browser window to upload your local files directly).*

## 5. Configure the Environment & Build

1. Create your `.env` file on the VM:
   ```bash
   nano .env
   ```
2. Paste your variables:
   ```env
   TELOXIDE_TOKEN="your_token_here"
   ADMIN_GROUP_ID="-123456789"
   PUBLIC_GROUP_ID="-100987654321"
   RUST_LOG=info
   ```
   *(Save and exit `nano` by pressing `Ctrl+O`, `Enter`, `Ctrl+X`)*.

3. Build the highly-optimized release version of the bot (this might take a few minutes):
   ```bash
   cargo build --release
   ```

## 6. Create a Systemd Background Service

You want your bot to run automatically in the background and restart if the VM reboots or if the bot crashes. We do this using `systemd`.

1. Open a new service file:
   ```bash
   sudo nano /etc/systemd/system/telegram-bot.service
   ```

2. Paste the following configuration (replace `YOUR_VM_USERNAME` with your actual username, which you can find by typing `whoami` in the terminal):

   ```ini
   [Unit]
   Description=Telegram Support Proxy Bot
   After=network.target

   [Service]
   Type=simple
   User=YOUR_VM_USERNAME
   WorkingDirectory=/home/YOUR_VM_USERNAME/telegram_bot
   ExecStart=/home/YOUR_VM_USERNAME/telegram_bot/target/release/telegram_bot
   Restart=always
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

3. Save and close (`Ctrl+O`, `Enter`, `Ctrl+X`).

## 7. Start the Bot

Run these commands to enable and start your new service:

```bash
# Reload systemd to recognize the new service
sudo systemctl daemon-reload

# Enable it to start automatically on system boot
sudo systemctl enable telegram-bot

# Start the bot right now
sudo systemctl start telegram-bot
```

### Checking the Logs

You can check if the bot is running successfully and view its logs anytime by running:
```bash
sudo journalctl -u telegram-bot -f
```
