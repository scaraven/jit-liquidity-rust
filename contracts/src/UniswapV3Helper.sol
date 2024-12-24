// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "@uniswap/v3-periphery/contracts/interfaces/ISwapRouter.sol";
import "@uniswap/v3-periphery/contracts/interfaces/INonfungiblePositionManager.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import {UniswapV3HelperInterface} from "./UniswapV3HelperInterface.sol";

contract UniswapV3Helper is Ownable, UniswapV3HelperInterface {
    ISwapRouter public immutable swapRouter;
    INonfungiblePositionManager public immutable positionManager;

    constructor(address _swapRouter, address _positionManager) Ownable(msg.sender) {
        swapRouter = ISwapRouter(_swapRouter);
        positionManager = INonfungiblePositionManager(_positionManager);
    }

    // Approve tokens for use in swaps and liquidity
    function approveToken(address token, uint256 amount) external onlyOwner {
        IERC20(token).approve(address(swapRouter), amount);
        IERC20(token).approve(address(positionManager), amount);
    }

    // Add liquidity to a Uniswap V3 pool
    function increaseLiquidity(
        address token0,
        address token1,
        uint24 fee,
        uint256 amount0,
        uint256 amount1,
        int24 tickLower,
        int24 tickUpper
    ) external onlyOwner returns (uint256 tokenId, uint128 liquidity, uint256 amount0Used, uint256 amount1Used) {
        IERC20(token0).transferFrom(msg.sender, address(this), amount0);
        IERC20(token1).transferFrom(msg.sender, address(this), amount1);

        INonfungiblePositionManager.MintParams memory params = INonfungiblePositionManager.MintParams({
            token0: token0,
            token1: token1,
            fee: fee,
            tickLower: tickLower,
            tickUpper: tickUpper,
            amount0Desired: amount0,
            amount1Desired: amount1,
            amount0Min: 0,
            amount1Min: 0,
            recipient: msg.sender,
            deadline: block.timestamp
        });

        return positionManager.mint(params);
    }

    // Perform a token swap
    function performSwap(address tokenIn, address tokenOut, uint24 fee, uint256 amountIn, uint256 amountOutMin)
        external
        onlyOwner
        returns (uint256 amountOut)
    {
        IERC20(tokenIn).transferFrom(msg.sender, address(this), amountIn);

        ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
            tokenIn: tokenIn,
            tokenOut: tokenOut,
            fee: fee,
            recipient: msg.sender,
            deadline: block.timestamp,
            amountIn: amountIn,
            amountOutMinimum: amountOutMin,
            sqrtPriceLimitX96: 0
        });

        return swapRouter.exactInputSingle(params);
    }

    // Remove liquidity from a Uniswap V3 pool
    function decreaseLiquidity(uint256 tokenId, uint128 liquidity)
        external
        onlyOwner
        returns (uint256 amount0, uint256 amount1)
    {
        INonfungiblePositionManager.DecreaseLiquidityParams memory params = INonfungiblePositionManager
            .DecreaseLiquidityParams({
            tokenId: tokenId,
            liquidity: liquidity,
            amount0Min: 0,
            amount1Min: 0,
            deadline: block.timestamp
        });

        return positionManager.decreaseLiquidity(params);
    }

    // Collect fees from a Uniswap V3 position
    function collectFees(uint256 tokenId) external onlyOwner returns (uint256 amount0, uint256 amount1) {
        INonfungiblePositionManager.CollectParams memory params = INonfungiblePositionManager.CollectParams({
            tokenId: tokenId,
            recipient: msg.sender,
            amount0Max: type(uint128).max,
            amount1Max: type(uint128).max
        });

        return positionManager.collect(params);
    }
}
