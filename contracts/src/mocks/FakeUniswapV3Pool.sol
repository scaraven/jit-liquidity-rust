// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract FakeUniswapV3Pool {
    address public token0;
    address public token1;
    uint24 public fee;

    constructor(address _token0, address _token1, uint24 _fee) {
        token0 = _token0;
        token1 = _token1;
        fee = _fee;
    }
}
