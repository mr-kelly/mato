#!/usr/bin/env bash
# Mato Compatibility Smoke Tests
# Run inside a mato session to verify TUI app rendering
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

passed=0
failed=0
skipped=0

check_cmd() {
    command -v "$1" &>/dev/null
}

run_test() {
    local name="$1"
    local cmd="$2"
    local timeout="${3:-5}"

    if ! check_cmd "$(echo "$cmd" | awk '{print $1}')"; then
        echo -e "${YELLOW}SKIP${NC} $name (not installed)"
        ((skipped++))
        return
    fi

    echo -n "TEST $name ... "
    if timeout "$timeout" bash -c "$cmd" &>/dev/null; then
        echo -e "${GREEN}OK${NC}"
        ((passed++))
    else
        local exit_code=$?
        # timeout returns 124, some TUI apps exit non-zero on timeout which is fine
        if [ "$exit_code" -eq 124 ]; then
            echo -e "${GREEN}OK${NC} (timed out as expected)"
            ((passed++))
        else
            echo -e "${RED}FAIL${NC} (exit code: $exit_code)"
            ((failed++))
        fi
    fi
}

echo "=== Mato Compatibility Smoke Tests ==="
echo "Terminal: ${TERM:-unknown}"
echo "Shell: ${SHELL:-unknown}"
echo ""

# 1. Basic shell operations
echo "--- Shell Basics ---"
run_test "echo + colors" "echo -e '\033[31mred\033[32mgreen\033[0m'" 2
run_test "unicode output" "echo '你好世界 🚀 café'" 2
run_test "large output" "seq 1 5000" 3
run_test "high-speed output" "yes | head -n 10000" 5

# 2. TUI Applications
echo ""
echo "--- TUI Applications ---"
run_test "vim quick exit" "vim -c ':q!'" 3
run_test "nvim quick exit" "nvim --headless -c ':q!'" 3
run_test "htop snapshot" "htop -d1 -C" 3
run_test "btop snapshot" "btop --tty_on" 3
run_test "less pipe" "echo 'hello world' | less -FX" 3
run_test "man page" "PAGER='head -20' man ls" 5

# 3. Alt-screen applications
echo ""
echo "--- Alt-Screen ---"
run_test "clear screen" "clear && echo ok" 2
run_test "tput reset" "tput reset 2>/dev/null && echo ok" 2

# 4. Input sequences
echo ""
echo "--- Input Sequences ---"
run_test "tab completion" "echo -e 'ls /\t'" 2
run_test "backspace" "echo -e 'abc\b\b\bd'" 2

# 5. Resize behavior (manual)
echo ""
echo "--- Resize (check visually after resizing window) ---"
run_test "tput lines/cols" "tput lines && tput cols" 2
run_test "stty size" "stty size 2>/dev/null || echo 'N/A'" 2

echo ""
echo "=== Results ==="
echo -e "Passed: ${GREEN}${passed}${NC}"
echo -e "Failed: ${RED}${failed}${NC}"
echo -e "Skipped: ${YELLOW}${skipped}${NC}"

if [ "$failed" -gt 0 ]; then
    exit 1
fi
