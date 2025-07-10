# Senator Budd Signal Chatbot Setup Guide

Complete instructions for setting up the Signal chatbot that responds as Senator Ted Budd to help Vice Admiral Mitch Bradley prepare for confirmation hearings.

## Prerequisites

1. **Phone Number** - You need a phone number that can receive SMS for Signal registration
2. **Anthropic API Key** - Sign up at https://console.anthropic.com for Claude access
3. **Computer** - macOS, Linux, or Windows with WSL

## Step 1: Install Signal CLI

### macOS (Homebrew)
```bash
brew install signal-cli
```

### Linux (Ubuntu/Debian)
```bash
# Add Signal's official software repository
wget -O- https://updates.signal.org/desktop/apt/keys.asc | gpg --dearmor > signal-desktop-keyring.gpg
cat signal-desktop-keyring.gpg | sudo tee -a /usr/share/keyrings/signal-desktop-keyring.gpg > /dev/null
echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/signal-desktop-keyring.gpg] https://updates.signal.org/desktop/apt xenial main' | sudo tee -a /etc/apt/sources.list.d/signal-xenial.list
sudo apt update && sudo apt install signal-desktop

# Install signal-cli
wget https://github.com/AsamK/signal-cli/releases/latest/download/signal-cli-*.tar.gz
tar xf signal-cli-*.tar.gz
sudo mv signal-cli-* /opt/signal-cli
sudo ln -sf /opt/signal-cli/bin/signal-cli /usr/local/bin/
```

### Windows (WSL)
Use the Linux instructions above in WSL.

## Step 2: Register Phone Number with Signal

**Important**: Use a phone number you control and can receive SMS on.

```bash
# Replace +1234567890 with your actual phone number
signal-cli -a +1234567890 register

# You'll receive an SMS with a verification code
# Enter it when prompted:
signal-cli -a +1234567890 verify 123456
```

**Verification successful when you see:**
```
You have successfully verified your phone number.
```

## Step 3: Test Signal CLI

```bash
# Test sending a message to yourself
signal-cli -a +1234567890 send +1234567890 -m "Test message"

# Test receiving (should show the test message)
signal-cli -a +1234567890 receive --json
```

## Step 4: Get Anthropic API Key

1. Go to https://console.anthropic.com
2. Sign up or log in
3. Navigate to API Keys section
4. Create a new API key
5. Copy the key (starts with `sk-ant-`)

## Step 5: Set Environment Variables

Create a `.env` file in the project root:

```bash
# Create .env file
cat > .env << EOF
ANTHROPIC_API_KEY=sk-ant-your-actual-api-key-here
SIGNAL_PHONE_NUMBER=+1234567890
DATABASE_URL=sqlite:chat_history.db
EOF
```

**Replace the values with:**
- `ANTHROPIC_API_KEY`: Your actual Anthropic API key
- `SIGNAL_PHONE_NUMBER`: The phone number you registered with Signal

## Step 6: Install Rust and Run the Bot

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Clone and Run
```bash
# If you haven't cloned yet
git clone https://github.com/soma9574/signal-ai.git
cd signal-ai

# Run the bot
cd backend
cargo run
```

**You should see startup logs like:**
```
🚀 Starting Senator Budd Signal Chatbot
📋 Environment check:
✅ ANTHROPIC_API_KEY found (length: 108)
✅ SIGNAL_PHONE_NUMBER found: +1234567890
📁 Database: sqlite:chat_history.db
✅ Database connected successfully
✅ LLM client initialized
✅ Signal client initialized
🔍 Testing Signal CLI availability...
✅ signal-cli found, version: 0.11.12
✅ Signal CLI test successful
🔄 Starting background Signal worker...
🌐 Server listening on 0.0.0.0:3000
📱 Ready to receive Signal messages!
```

## Step 7: Test the Bot

1. **Send a test message** from another phone to your registered number:
   ```
   "What are your thoughts on military readiness?"
   ```

2. **Check the bot logs** - you should see:
   ```
   📨 Received Signal message from +1987654321: What are your thoughts on military readiness?
   🤖 Generating LLM response...
   ✅ Generated response: As Senator from North Carolina, I believe...
   💾 Storing conversation in database...
   📤 Sending response via Signal...
   ✅ Sent Signal response to +1987654321
   ```

3. **You should receive a response** from Senator Budd persona within ~10 seconds

## Step 8: Health Check

Test that everything is working:

```bash
curl http://localhost:3000/health
```

Should return:
```json
{
  "status": "healthy",
  "signal_cli_available": true,
  "database_connected": true,
  "phone_number": "+1234567890"
}
```

## Usage Instructions for Admiral Bradley

Once setup is complete:

1. **Text the bot** at the registered phone number
2. **Ask questions** like:
   - "What are your thoughts on SOCOM's current priorities?"
   - "How do you view the confirmation process?"
   - "What questions might come up about military spending?"

3. **Receive responses** from "Senator Budd" within 10 seconds
4. **All conversations** are saved automatically for review

## Troubleshooting

### Signal CLI Issues

**Error: `signal-cli not found`**
```bash
# Check if installed
which signal-cli
signal-cli --version

# If not found, reinstall using instructions above
```

**Error: `User +1234567890 is not registered`**
```bash
# Re-register your number
signal-cli -a +1234567890 register
# Follow SMS verification steps
```

**Error: `Failed to send message: Untrusted Identity`**
```bash
# Trust all identities (for development)
signal-cli -a +1234567890 listIdentities
signal-cli -a +1234567890 trust -a
```

### Bot Issues

**Error: `ANTHROPIC_API_KEY not set`**
- Check your `.env` file exists and has the correct API key
- Ensure you're running from the `backend/` directory

**Error: `Database connection failed`**
- Ensure SQLite is available (usually built-in)
- Check file permissions in the project directory

**No responses to messages:**
1. Check bot logs for error messages
2. Verify Signal CLI can send/receive manually
3. Check `/health` endpoint
4. Ensure bot phone number is registered correctly

### Getting Help

If you encounter issues:

1. **Check the logs** - the bot provides detailed emoji-coded logging
2. **Test Signal CLI manually** - ensure basic send/receive works
3. **Verify environment variables** - check `.env` file
4. **Check health endpoint** - `curl http://localhost:3000/health`

## Production Deployment

For Railway deployment:
1. Push code to GitHub
2. Connect Railway to your repository
3. Set environment variables in Railway dashboard:
   - `ANTHROPIC_API_KEY`
   - `SIGNAL_PHONE_NUMBER`
4. Install signal-cli in Railway environment (may require Docker)

The SQLite database will persist with your Railway deployment automatically. 