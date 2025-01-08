### JIT-Liqudity-Rust
A rust bot using Alloy-rs, Revm, and Alloy-mev in order to provide just-in-time (JIT) liquidity on UniswapV3.

### Setup
To run the bot, you require cargo which can be installed from the Rust's official website [here](https://www.rust-lang.org/tools/install). The solidity contracts use forge and anvil which can be installed via foundryup found [here](https://book.getfoundry.sh/getting-started/installation). Additionally, you need to create a .env file which contains the following keys:

PRIVATE_KEY, INFURA_URL, ANVIL_ENDPOINT


### TODO
1. Expand filter by simulating transaction with REVM to figure out whether we have an internal call to the UniswapV3 router (logs?)
    - This can be used to catch value provided by protocols which deploy DEFI strategies on-chain
2. Allow bundling of multiple swaps to the same pool
3. Migrate to using MEV-Share event stream and figure out whether it targets public mempool transactions
4. Create end-end simulation example with integration tests
5. Health checker in rust, pulls the switch if something is going drastically wrong
6. Output formatter, allows user to see what the bot is doing

### Capabilities
The bot should operate as follows:
1. Monitor public mempool transactions and use a shallow filter to choose sandwichable transactions
2. Take a promising transaction and create a bundle which calls our Executor.sol contract
3. Check whether the bundle is profitable using REVM and inspecting final states
4. Submit the bundle to MEV-Share
5. Execute a post Execution Manager which ensures that nothing has gone wrong

Executor.sol is a file which is deployed on Ethereum and handles much of the trading logic including executing UniswapV3 liquidity positions
