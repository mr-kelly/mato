#!/bin/bash
# Test runner for terminal persistence tests

set -e

echo "=== MATO Terminal Persistence Tests ==="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Run unit tests
echo -e "${YELLOW}Running unit tests...${NC}"
cargo test --test terminal_persistence_tests
echo -e "${GREEN}✓ Unit tests passed${NC}"
echo

# Check if daemon is running
DAEMON_RUNNING=false
if pgrep -f "mato.*daemon" > /dev/null; then
    DAEMON_RUNNING=true
    echo -e "${GREEN}✓ Daemon is running${NC}"
else
    echo -e "${YELLOW}⚠ Daemon is not running${NC}"
    echo "  Starting daemon for integration tests..."
    
    # Start daemon in background
    cargo run -- --daemon --foreground > /tmp/mato-test-daemon.log 2>&1 &
    DAEMON_PID=$!
    
    # Wait for daemon to start
    sleep 2
    
    if pgrep -f "mato.*daemon" > /dev/null; then
        echo -e "${GREEN}✓ Daemon started (PID: $DAEMON_PID)${NC}"
        DAEMON_RUNNING=true
    else
        echo -e "${RED}✗ Failed to start daemon${NC}"
        cat /tmp/mato-test-daemon.log
        exit 1
    fi
fi

# Run integration tests if daemon is running
if [ "$DAEMON_RUNNING" = true ]; then
    echo
    echo -e "${YELLOW}Running integration tests...${NC}"
    
    # Update socket path to match daemon
    SOCKET_PATH="$HOME/.local/state/mato/daemon.sock"
    
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${GREEN}✓ Daemon socket found: $SOCKET_PATH${NC}"
        
        # Run ignored tests (integration tests)
        cargo test --test daemon_persistence_tests -- --ignored --test-threads=1
        
        echo -e "${GREEN}✓ Integration tests passed${NC}"
    else
        echo -e "${RED}✗ Daemon socket not found: $SOCKET_PATH${NC}"
        exit 1
    fi
fi

echo
echo -e "${GREEN}=== All tests passed! ===${NC}"

# Cleanup
if [ -n "$DAEMON_PID" ]; then
    echo
    echo "Stopping test daemon..."
    kill $DAEMON_PID 2>/dev/null || true
fi
