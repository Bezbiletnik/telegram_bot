# Updating Your Deployed Telegram Bot

Whenever you make changes to your local bot code, you will need to push those changes to GitHub and pull them onto your Google Cloud VM. Here is the complete workflow.

## Phase 1: On Your Local Machine (Pushing to GitHub)

First, send the new code from your local computer to your GitHub repository. Open your local terminal in the `telegram_bot` directory and run:

1. **Stage the changes:** This tells Git to prepare all modified files.
   ```bash
   git add .
   ```

2. **Commit the changes:** Package the changes with a descriptive message.
   ```bash
   git commit -m "Update bot logic"
   ```

3. **Push to GitHub:** Upload the code to your repository.
   ```bash
   git push
   ```
   *(Note: Depending on your git setup, you might need to specify the branch, e.g., `git push origin main` or `git push origin master`)*

---

## Phase 2: On Your Google Cloud VM (Pulling and Restarting)

Now that GitHub has your latest code, you need to update the VM. 
*Note: Because you originally cloned the repository on the VM, Git already knows the exact URL to pull the code from (stored as "origin").*

1. **Connect to the VM:** 
   Go to the [Google Cloud Console](https://console.cloud.google.com/), navigate to **Compute Engine > VM instances**, and click the **SSH** button next to your VM.

2. **Navigate to the project directory:**
   ```bash
   cd telegram_bot
   ```

3. **Download the new code:** 
   Fetch and merge the newest code from GitHub.
   ```bash
   git pull
   ```

4. **Recompile the bot:**
   Build the new version of the Rust code into an executable. This might take a minute or two.
   ```bash
   cargo build --release
   ```

5. **Restart the background service:**
   This command stops the old version of the bot and starts the newly compiled one.
   ```bash
   sudo systemctl restart telegram-bot
   ```

6. **Verify the bot is running (Optional):**
   Check the live logs to ensure the bot started successfully without errors.
   ```bash
   sudo journalctl -u telegram-bot -f
   ```
   *(Press `Ctrl+C` when you are done looking at the logs).*

You're done! The live bot is now running your newest code.
