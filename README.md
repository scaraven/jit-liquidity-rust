# JIT-Liquidity-Rust
A Rust-based bot utilizing Alloy-rs and REVM to provide just-in-time (JIT) liquidity on UniswapV3.

## Setup
To run the bot, ensure you have the following tools installed:

1. **Rust & Cargo**: Install Cargo from Rust's official website: [Install Rust](https://www.rust-lang.org/tools/install).
2. **Forge & Anvil**: Install Forge and Anvil using Foundry's installation guide: [Foundry Installation](https://book.getfoundry.sh/getting-started/installation).
3. **Environment Variables**: Create a `.env` file with the following keys:
   - `PRIVATE_KEY`: Your Ethereum private key.
   - `ANVIL_ENDPOINT`: URL for the Anvil RPC endpoint.
   - `INFURA_URL`: Ethereum RPC URL (can be any valid Ethereum endpoint, not just Infura).

## Running the Bot
**TODO**: Instructions for running the bot are currently under development.

## Testing

### Environment Setup
The following environment variables are required for testing:

- `INFURA_URL`: RPC URL to fork the Ethereum blockchain.
- `INFURA_WS_URL`: Websocket URL to listen to public transactions
- `INFURA_URL_BLOCK`: Block number for forking the blockchain (default: `21431100`).
- `TEST_PRIVATE_KEY`: Private key for testing (default: Anvil's pre-generated key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`).
- `ETHERSCAN_API_KEY` (optional): Enables Foundry to provide detailed debugging messages.

**Recommended**: Create a `.env` file with the above variables. Then, create a symbolic link to this file in the `contracts/` directory so that Foundry can access the environment variables:

```bash
cd contracts
ln -s ../.env .env
```

### Running Tests
This repository includes unit tests for both:

1. Solidity contracts deployed on Ethereum.
2. Rust library containing the main bot logic.

Both require an RPC URL. Use the following commands to run tests:

```bash
export $INFURA_URL="<your rpc url>"
export $INFURA_URL_BLOCK=21431100
./run-tests
cd contracts
forge test -vvv
```

## TODO List
1. Expand the transaction filter to simulate transactions with REVM and identify internal calls to the UniswapV3 router (e.g., through logs).
   - This will help capture value from protocols deploying DeFi strategies on-chain.
2. Enable bundling of multiple swaps for the same pool.
3. Integrate with MEV-Share event stream and determine if it targets public mempool transactions.
4. Create an end-to-end simulation example with integration tests.
5. Add a health checker in Rust to stop the bot if critical issues arise.
6. Implement an output formatter for user-friendly bot activity logs.
7. Enhance `Executor.sol` to provide liquidity at different ticks based on sandwich transaction data.

## Capabilities
The bot is designed to:

1. Monitor public mempool transactions and apply a shallow filter to identify sandwichable transactions.
2. Select a promising transaction and create a bundle that calls the `Executor.sol` contract.
3. Simulate the bundle with REVM to verify profitability by inspecting final states.
4. Submit profitable bundles to MEV-Share.
5. Run a post-execution manager to ensure nothing went wrong.

### `Executor.sol`
`Executor.sol` is a Solidity contract deployed on Ethereum. It handles the core trading logic, including managing UniswapV3 liquidity positions.
