#!/bin/bash

# ---------- Read .env file ----------
if [ -f .env ]; then
    export $(cat .env | xargs)
fi

# ---------- Set up arguments ----------
USAGE_MESSAGE="Usage: $0 --fu <fork-url> --fbn <fork-block-number> --tn <test-name>"

# INFURA_URL
URL=$INFURA_URL

if [[ "$@" == *"--fu"* ]]; then
    URL=$(echo "$@" | grep -oP -- '--fu \K[^ ]+')
fi


if [ -z "$URL" ]; then
    echo $USAGE_MESSAGE
    exit 1
fi

# INFURA_URL_BLOCK
BLOCK_NUMBER=$INFURA_URL_BLOCK

if [[ "$@" == *"--fbn"* ]]; then
    BLOCK_NUMBER=$(echo "$@" | grep -oP -- '--fbn \K[^ ]+')
fi

if [ -z "$BLOCK_NUMBER" ]; then
    echo $USAGE_MESSAGE
    exit 1
fi

# TEST_NAME
if [[ "$@" == *"--tn"* ]]; then
    TEST_NAME=$(echo "$@" | grep -oP -- '--tn \K[^ ]+')
fi

# ---------- Print environment ----------
echo "Environment: "
echo "  URL: $URL"
echo "  BLOCK_NUMBER: $BLOCK_NUMBER"
if [ -z "$TEST_NAME" ]; then
    echo "  TEST_NAME: All tests"
else
    echo "  TEST_NAME: $TEST_NAME"
fi

# ---------- Start anvil ----------
anvil --fork-url $URL --fork-block-number $BLOCK_NUMBER > /dev/null 2>&1 &
ANVIL_PID=$!

echo "Anvil started with PID $ANVIL_PID"

# Function to clean up (kill anvil) on exit or error
cleanup() {
    echo "Stopping Anvil..."
    kill $ANVIL_PID
    wait $ANVIL_PID 2>/dev/null
}
trap cleanup EXIT

# Wait for Anvil to start
echo "Waiting for Anvil to be ready..."
for i in {1..10}; do
    if curl -s http://localhost:8545 > /dev/null; then
        echo "Anvil is ready."
        break
    fi
    sleep 1
done

# Check if Anvil started successfully
if ! curl -s http://localhost:8545 > /dev/null; then
    echo "Anvil failed to start. Exiting."
    exit 1
fi

# ---------- Run cargo test ----------
echo "Running cargo test..."
echo "Command: cargo test $TEST_NAME -- --test-threads=1"

if cargo test $TEST_NAME -- --test-threads=1; then
    echo "Tests completed successfully."
else
    echo "Tests failed."
    exit 1
fi
exit 0

# Clean up is handled by the trap
