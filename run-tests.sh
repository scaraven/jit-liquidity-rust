#!/bin/bash

# Start anvil and run in the background
anvil --fork-url https://mainnet.infura.io/v3/b826cf4e514847b792b840ff1e29fd11@21431100 > /dev/null 2>&1 &
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

# Run cargo test
echo "Running cargo test..."
if cargo test -- --test-threads=1; then
    echo "Tests completed successfully."
else
    echo "Tests failed."
    exit 1
fi

# Clean up is handled by the trap
