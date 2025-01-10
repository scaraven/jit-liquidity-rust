// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IUniswapV3MintCallback} from "@uniswap/v3-core/contracts/interfaces/callback/IUniswapV3MintCallback.sol";

interface IExecutor is IUniswapV3MintCallback {
    function execute(address pool) external;
    function finish() external;
    function withdraw(address[] calldata tokens) external;

    function setFundManager(address _fundManager) external;
}
