// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

interface UniswapV3HelperInterface {
    function approveToken(address token, uint256 amount) external;
    function increaseLiquidity(
        address token0,
        address token1,
        uint24 fee,
        uint256 amount0,
        uint256 amount1,
        int24 tickLower,
        int24 tickUpper
    ) external returns (uint256 tokenId, uint128 liquidity, uint256 amount0Used, uint256 amount1Used);
}
