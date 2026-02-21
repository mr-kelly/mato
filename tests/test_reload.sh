#!/bin/bash
# Test config reload on SIGHUP

set -e

CONFIG_PATH="$HOME/.config/mato/config.toml"
LOG_PATH="$HOME/.local/state/mato/daemon.log"

echo "=== Testing Config Reload on SIGHUP ==="
echo

# Get daemon PID
PID=$(cat ~/.local/state/mato/daemon.pid 2>/dev/null || echo "")

if [ -z "$PID" ]; then
    echo "❌ Daemon not running. Start it with 'mato' first."
    exit 1
fi

echo "✅ Daemon PID: $PID"
echo

# Show current config
echo "📄 Current config:"
cat "$CONFIG_PATH" 2>/dev/null || echo "emulator = \"vte\""
echo

# Change config
echo "🔧 Changing emulator to 'vt100'..."
mkdir -p "$(dirname "$CONFIG_PATH")"
echo 'emulator = "vt100"' > "$CONFIG_PATH"
echo

# Send SIGHUP
echo "📡 Sending SIGHUP to daemon..."
kill -HUP "$PID"
sleep 1
echo

# Check log
echo "📝 Daemon log (last 5 lines):"
tail -5 "$LOG_PATH"
echo

# Verify reload
if grep -q "Configuration reloaded: emulator=vt100" "$LOG_PATH"; then
    echo "✅ Config reload successful!"
else
    echo "⚠️  Config reload not detected in log"
fi

echo
echo "=== Test Complete ==="
