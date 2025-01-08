// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {INonfungiblePositionManager} from "@uniswap/v3-periphery/contracts/interfaces/INonfungiblePositionManager.sol";
import {IUniswapV3Pool} from "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";
import {IERC20Token} from "@interfaces/IERC20Token.sol";
import {IFundManager} from "@interfaces/IFundManager.sol";
import {IExecutor} from "@interfaces/IExecutor.sol";

import {FullMath} from "@uniswap/v3-core/contracts/libraries/FullMath.sol";
import {FixedPoint96} from "@uniswap/v3-core/contracts/libraries/FixedPoint96.sol";
import {SafeCast} from "@uniswap/v3-core/contracts/libraries/SafeCast.sol";

struct Position {
    uint256 tokenId;
    uint128 liquidity;
    uint256 amount0;
    uint256 amount1;
    address token0;
    address token1;
}

struct MetricParams {
    address token0;
    address token1;
    uint256 amount0;
    uint256 amount1;
    uint24 fee;
    int24 tickLower;
    int24 tickUpper;
}

// Is a proxy contract that allows the owner to execute sandwich trades
contract Executor is IExecutor, Ownable {
    using SafeCast for uint256;

    IFundManager public fundManager;
    Whitelist public whitelist;
    INonfungiblePositionManager public manager;

    mapping(address => mapping(int24 => mapping(int24 => uint256))) tokenIds;

    uint256 private execution_bit;
    Position private position;

    modifier notExecuting() {
        require(execution_bit == 0, "EXECUTOR: Currently executing");
        execution_bit = 1;
        _;
    }

    modifier Executing() {
        require(execution_bit == 1, "EXECUTOR: Not executing");
        execution_bit = 0;
        _;
    }

    constructor(address _owner, address _fundManager, address _whitelist, address _manager) Ownable(_owner) {
        fundManager = IFundManager(_fundManager);
        whitelist = Whitelist(_whitelist);
        manager = INonfungiblePositionManager(_manager);
    }

    function execute(address pool) external override notExecuting onlyOwner {
        require(whitelist.check_whitelist(pool), "EXECUTOR: Pool not whitelisted");

        // Calculate metrics
        MetricParams memory metrics = calc_metrics(pool);

        // Create array of tokens
        address[] memory tokens = new address[](2);
        tokens[0] = metrics.token0;
        tokens[1] = metrics.token1;

        // Start benchmark
        fundManager.start_benchmark(address(this), tokens);

        // If we already have a tokenId for this pool, increase liquidity
        uint256 tokenIdPossible = tokenIds[pool][metrics.tickLower][metrics.tickUpper];
        if (tokenIdPossible != 0) {
            (uint128 liquidity, uint256 amount0In, uint256 amount1In) =
                increase(tokenIdPossible, metrics.amount0, metrics.amount1);
            position = Position(tokenIdPossible, liquidity, amount0In, amount1In, metrics.token0, metrics.token1);
        } else {
            // Otherwise, mint a new position
            (uint256 tokenId, uint128 liquidity, uint256 amount0In, uint256 amount1In) = mint(
                metrics.token0,
                metrics.token1,
                metrics.amount0,
                metrics.amount1,
                metrics.tickLower,
                metrics.tickUpper,
                metrics.fee
            );
            tokenIds[pool][metrics.tickLower][metrics.tickUpper] = tokenId;
            position = Position(tokenId, liquidity, amount0In, amount1In, metrics.token0, metrics.token1);
        }
    }

    function calc_metrics(address pool) public view returns (MetricParams memory) {
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
        {
            int24 tick;
            (sqrtPriceX96, tick,,,,,) = pool_contract.slot0();

            // Get the pool contract spacing
            int24 spacing = pool_contract.tickSpacing();

            (tickLower, tickUpper) = calculate_tick_bounds(tick, spacing);
        }

        MetricParams memory metrics = MetricParams(token0, token1, amount0Max, amount1Max, fee, tickLower, tickUpper);
        return metrics;
    }

    function calculate_tick_bounds(int24 tick, int24 spacing)
        internal
        pure
        returns (int24 tickLower, int24 tickUpper)
    {
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

    function mint(
        address token0,
        address token1,
        uint256 amount0,
        uint256 amount1,
        int24 tickLower,
        int24 tickUpper,
        uint24 fee
    ) internal returns (uint256 tokenId, uint128 liquidity, uint256 amount0In, uint256 amount1In) {
        // Supply liquidity to UniswapV3 pool
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
            recipient: address(this),
            deadline: block.timestamp
        });

        (tokenId, liquidity, amount0In, amount1In) = manager.mint(params);
    }

    function increase(uint256 tokenId, uint256 amount0, uint256 amount1)
        internal
        returns (uint128 liquidity, uint256 amount0In, uint256 amount1In)
    {
        INonfungiblePositionManager.IncreaseLiquidityParams memory params = INonfungiblePositionManager
            .IncreaseLiquidityParams({
            tokenId: tokenId,
            amount0Desired: amount0,
            amount1Desired: amount1,
            amount0Min: 0,
            amount1Min: 0,
            deadline: block.timestamp
        });

        (liquidity, amount0In, amount1In) = manager.increaseLiquidity(params);
    }

    function finish() external override Executing onlyOwner {
        // Fetch position
        Position memory _position = position;
        INonfungiblePositionManager.CollectParams memory collectParams = INonfungiblePositionManager.CollectParams({
            tokenId: _position.tokenId,
            recipient: address(this),
            amount0Max: type(uint128).max,
            amount1Max: type(uint128).max
        });

        manager.collect(collectParams);

        // Decrease all of the liquidity
        INonfungiblePositionManager.DecreaseLiquidityParams memory decreaseParams = INonfungiblePositionManager
            .DecreaseLiquidityParams({
            tokenId: _position.tokenId,
            liquidity: _position.liquidity,
            amount0Min: _position.amount0,
            amount1Min: _position.amount1,
            deadline: block.timestamp
        });

        manager.decreaseLiquidity(decreaseParams);
        // Finish benchmark
        address[] memory tokens = new address[](2);
        tokens[0] = _position.token0;
        tokens[1] = _position.token1;

        require(fundManager.end_benchmark(address(this), tokens), "EXECUTOR: Benchmark failed");
    }

    function setup(address[] calldata tokens) external override onlyOwner {
        for (uint256 i = 0; i < tokens.length; i++) {
            IERC20Token token = IERC20Token(tokens[i]);
            token.approve(address(manager), type(uint256).max);
        }
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
