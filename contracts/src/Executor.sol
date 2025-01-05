// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Whitelist} from "./Whitelist.sol";
import {INonfungiblePositionManager} from "@uniswap/v3-periphery/contracts/interfaces/INonfungiblePositionManager.sol";
import {IUniswapV3Pool} from "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";
import {IERC20Token} from "./interfaces/IERC20Token.sol";

struct Position {
    uint256 tokenId;
    uint128 liquidity;
    uint256 amount0;
    uint256 amount1;
    address token0;
    address token1;
}

// Is a proxy contract that allows the owner to execute sandwich trades
contract Executor is Ownable {
    IFundManager public fundManager;
    Whitelist public whitelist;
    INonfungiblePositionManager public manager;

    mapping(address => mapping(int24 => uint256)) tokenIds;

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

    function execute(address pool, uint128 amount, int24 tick) external notExecuting onlyOwner {
        require(whitelist.check_whitelist(pool), "EXECUTOR: Pool not whitelisted");

        // Fetch pool
        IUniswapV3Pool pool_contract = IUniswapV3Pool(pool);
        address token0 = pool_contract.token0();
        address token1 = pool_contract.token1();

        address[] memory tokens = new address[](2);
        tokens[0] = token0;
        tokens[1] = token1;

        // Start benchmark
        fundManager.start_benchmark(address(this), tokens);

        // If we already have a tokenId for this pool, increase liquidity
        uint256 tokenIdPossible = tokenIds[pool][tick];
        if (tokenIdPossible != 0) {
            (uint128 liquidity, uint256 amount0, uint256 amount1) = increase(tokenIdPossible, amount);
            position = Position(tokenIdPossible, liquidity, amount0, amount1, token0, token1);
        } else {
            // Otherwise, mint a new position
            (uint256 tokenId, uint128 liquidity, uint256 amount0, uint256 amount1) = mint(token0, token1, amount, tick);
            tokenIds[pool][tick] = tokenId;
            position = Position(tokenId, liquidity, amount0, amount1, token0, token1);
        }
    }

    function mint(address token0, address token1, uint128 amount, int24 tick)
        internal
        returns (uint256 tokenId, uint128 liquidity, uint256 amount0, uint256 amount1)
    {
        // Supply liquidity to UniswapV3 pool
        INonfungiblePositionManager.MintParams memory params = INonfungiblePositionManager.MintParams({
            token0: token0,
            token1: token1,
            fee: 3000,
            tickLower: tick,
            tickUpper: tick,
            amount0Desired: amount,
            amount1Desired: amount,
            amount0Min: amount,
            amount1Min: amount,
            recipient: address(this),
            deadline: block.timestamp
        });

        (tokenId, liquidity, amount0, amount1) = manager.mint(params);
    }

    function increase(uint256 tokenId, uint256 amount)
        internal
        returns (uint128 liquidity, uint256 amount0, uint256 amount1)
    {
        INonfungiblePositionManager.IncreaseLiquidityParams memory params = INonfungiblePositionManager
            .IncreaseLiquidityParams({
            tokenId: tokenId,
            amount0Desired: amount,
            amount1Desired: amount,
            amount0Min: amount,
            amount1Min: amount,
            deadline: block.timestamp
        });

        (liquidity, amount0, amount1) = manager.increaseLiquidity(params);
    }

    function finish() external Executing onlyOwner {
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

    function setup(address[] calldata tokens) external onlyOwner {
        for (uint256 i = 0; i < tokens.length; i++) {
            IERC20Token token = IERC20Token(tokens[i]);
            token.approve(address(manager), type(uint256).max);
        }
    }

    function withdraw(address[] calldata tokens) external onlyOwner {
        for (uint256 i = 0; i < tokens.length; i++) {
            IERC20Token token = IERC20Token(tokens[i]);
            token.transfer(owner(), token.balanceOf(address(this)));
        }
    }
}
