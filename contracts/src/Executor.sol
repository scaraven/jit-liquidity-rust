// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {IUniswapV3Pool} from "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";
import {IERC20Token} from "@interfaces/IERC20Token.sol";
import {IFundManager} from "@interfaces/IFundManager.sol";
import {IExecutor} from "@interfaces/IExecutor.sol";

import {LiquidityAmounts} from "@uniswap/v3-periphery/contracts/libraries/LiquidityAmounts.sol";
import {TickMath} from "@uniswap/v3-core/contracts/libraries/TickMath.sol";

import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Executor is IExecutor, Ownable {
    using SafeERC20 for IERC20;

    error BenchMarkFailure();

    // Constants for rounding and margin of error
    uint128 private constant BASE = 100;
    uint128 private constant PERCENTAGE = 90;

    struct MetricParams {
        address pool;
        address token0;
        address token1;
        uint24 fee;
        int24 tick;
        int24 tickLower;
        int24 tickUpper;
        uint128 liquidity;
    }

    error UnauthorizedPool(address expected, address actual);

    IFundManager public fundManager;
    Whitelist public whitelist;

    uint256 private execution_bit;
    MetricParams public metrics;

    modifier notExecuting() {
        require(execution_bit == 0, "EXECUTOR: Currently executing");
        execution_bit = 1;
        _;
    }

    modifier isExecuting() {
        require(execution_bit == 1, "EXECUTOR: execute() has not been called");
        _;
    }

    modifier Executing() {
        require(execution_bit == 1, "EXECUTOR: Not executing");
        execution_bit = 0;
        _;
    }

    constructor(address _owner, address _fundManager, address _whitelist) Ownable(_owner) {
        fundManager = IFundManager(_fundManager);
        whitelist = Whitelist(_whitelist);
    }

    function execute(address pool) external override notExecuting onlyOwner {
        require(whitelist.checkWhitelist(pool), "EXECUTOR: Pool not whitelisted");

        // Calculate metrics
        MetricParams memory _metrics = calcMetrics(pool);
        // Write to storage
        metrics = _metrics;

        // Create array of tokens
        address[] memory tokens = new address[](3);
        tokens[0] = _metrics.token0;
        tokens[1] = _metrics.token1;

        // Start benchmark
        fundManager.startBenchmark(address(this), tokens);

        // Mint liquidity to pool and create callback to ourselves
        IUniswapV3Pool(pool).mint(address(this), metrics.tickLower, metrics.tickUpper, metrics.liquidity, "");
    }

    function uniswapV3MintCallback(uint256 amount0Owed, uint256 amount1Owed, bytes calldata)
        external
        override
        isExecuting
    {
        // Fetch current position into memory
        MetricParams memory position = metrics;

        require(msg.sender == position.pool, UnauthorizedPool(position.pool, msg.sender));

        // Transfer tokens to pool
        IERC20(position.token0).safeTransfer(position.pool, amount0Owed);
        IERC20(position.token1).safeTransfer(position.pool, amount1Owed);
    }

    function calcMetrics(address pool) public view returns (MetricParams memory) {
        // Fetch pool
        IUniswapV3Pool pool_contract = IUniswapV3Pool(pool);
        address token0 = pool_contract.token0();
        address token1 = pool_contract.token1();

        // Check how much we are able to add
        uint256 amount0Max = IERC20Token(token0).balanceOf(address(this));
        uint256 amount1Max = IERC20Token(token1).balanceOf(address(this));

        uint24 fee = pool_contract.fee();
        int24 tickLower;
        int24 tickUpper;
        uint160 sqrtPriceX96;
        int24 tick;
        {
            (sqrtPriceX96, tick,,,,,) = pool_contract.slot0();

            // Get the pool contract spacing
            int24 spacing = pool_contract.tickSpacing();

            (tickLower, tickUpper) = calculateTickBounds(tick, spacing);
        }

        // Given tick range, calculate the amount of liquidity we can add
        uint128 liquidity = LiquidityAmounts.getLiquidityForAmounts(
            TickMath.getSqrtRatioAtTick(tick),
            TickMath.getSqrtRatioAtTick(tickLower),
            TickMath.getSqrtRatioAtTick(tickUpper),
            amount0Max,
            amount1Max
        );

        MetricParams memory _metrics =
            MetricParams(pool, token0, token1, fee, tick, tickLower, tickUpper, liquidity * PERCENTAGE / BASE);
        return _metrics;
    }

    function calculateTickBounds(int24 tick, int24 spacing) internal pure returns (int24 tickLower, int24 tickUpper) {
        // Calculate lower and upper ticks
        // If tick is not on a boundary, then choose upper and lower bound
        int24 remain = tick % spacing;
        if (remain == 0) {
            tickLower = tick - spacing;
            tickUpper = tick + spacing;
        } else {
            tickLower = tick - remain;
            tickUpper = tick + (spacing - remain);
        }
    }

    function finish() external override Executing onlyOwner {
        // Fetch position
        MetricParams memory _position = metrics;

        // Burn liquidity and then collect tokens
        IUniswapV3Pool(_position.pool).burn(_position.tickLower, _position.tickUpper, _position.liquidity);
        IUniswapV3Pool(_position.pool).collect(
            address(this), _position.tickLower, _position.tickUpper, type(uint128).max, type(uint128).max
        );

        // End benchmark
        address[] memory tokens = new address[](3);
        tokens[0] = _position.token0;
        tokens[1] = _position.token1;
        require(fundManager.endBenchmark(address(this), tokens), BenchMarkFailure());
    }

    function withdraw(address[] calldata tokens) external override onlyOwner {
        for (uint256 i = 0; i < tokens.length; i++) {
            IERC20Token token = IERC20Token(tokens[i]);
            token.transfer(owner(), token.balanceOf(address(this)));
        }
    }

    function setFundManager(address _fundManager) external override onlyOwner {
        fundManager = IFundManager(_fundManager);
    }
}
