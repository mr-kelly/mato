#!/bin/bash
# Test MATO with complex TUI applications

echo "Testing MATO Terminal Emulation"
echo "================================"
echo ""

# Test 1: Basic colors
echo "Test 1: ANSI Colors"
echo -e "\033[31mRed\033[0m \033[32mGreen\033[0m \033[33mYellow\033[0m \033[34mBlue\033[0m"
echo -e "\033[1mBold\033[0m \033[4mUnderline\033[0m \033[7mReverse\033[0m"
echo ""

# Test 2: Cursor movement
echo "Test 2: Cursor Movement"
echo -e "Line 1\nLine 2\nLine 3"
echo -e "\033[2A\033[10CInserted"
echo ""

# Test 3: Clear operations
echo "Test 3: Clear Operations"
echo "Before clear..."
sleep 1
echo -e "\033[2J\033[H"
echo "After clear (should be at top)"
echo ""

# Test 4: Box drawing
echo "Test 4: Box Drawing Characters"
echo "┌─────────┐"
echo "│  Box    │"
echo "└─────────┘"
echo ""

# Test 5: Progress bar
echo "Test 5: Progress Bar"
for i in {1..10}; do
    echo -ne "\r["
    for j in $(seq 1 $i); do echo -n "="; done
    for j in $(seq $i 9); do echo -n " "; done
    echo -n "] $((i*10))%"
    sleep 0.1
done
echo ""
echo ""

echo "All tests completed!"
echo "If you can see all the above correctly, terminal emulation is working well."
