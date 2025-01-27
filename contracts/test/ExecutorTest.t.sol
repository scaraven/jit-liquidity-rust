// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";

import {Whitelist} from "@core/Whitelist.sol";
import {FundManager} from "@core/FundManager.sol";
import {Oracle} from "@core/Oracle.sol";
import {Executor} from "@core/Executor.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ISwapRouter} from "@uniswap/v3-periphery/contracts/interfaces/ISwapRouter.sol";

contract ExecutorTest is Test {
    Whitelist whitelist;
    FundManager fundManager;
    Executor executor;
    Oracle oracle;

    address public alice;

    ISwapRouter public swapRouter;

    address constant POOL_ADDR = address(0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640);
    address constant WETH = address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);
    address constant WBTC = address(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);

    // Chainlink oracles
    address constant BTC_USD = address(0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c);
    address constant ETH_USD = address(0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419);
    address constant USDC_USD = address(0x8fFfFfd4AfB6115b954Bd326cbe7B4BA576818f6);

    address constant WETH_WHALE = address(0xF04a5cC80B1E94C69B48f5ee68a08CD2F09A7c3E);
    address constant USDC_WHALE = address(0x37305B1cD40574E4C5Ce33f8e8306Be057fD7341);

    address constant FACTORY = address(0x1F98431c8aD98523631AE4a59f267346ea31F984);

    function setUp() public {
        vm.createSelectFork(vm.envString("RPC_URL"), vm.envUint("RPC_TEST_URL_BLOCK"));
        swapRouter = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);
        alice = makeAddr("alice");

        // Initialize the Whitelist contract
        whitelist = new Whitelist(address(this), FACTORY);
        whitelist.addWhitelist(POOL_ADDR);

        // Initialize the Oracle contract
        oracle = new Oracle();
        oracle.setFeed(address(0), ETH_USD);
        oracle.setFeed(WETH, ETH_USD);
        oracle.setFeed(USDC, USDC_USD);

        fundManager = new FundManager(address(this));
        // Setup oracles
        fundManager.setOracle(address(oracle));

        executor = new Executor(address(this), address(fundManager), address(whitelist));
        // Transfer ownership to executor
        fundManager.transferOwnership(address(executor));

        // Transfer some WETH and ETH to the executor
        IERC20(WETH).balanceOf(WETH_WHALE);
        IERC20(USDC).balanceOf(USDC_WHALE);

        vm.startPrank(WETH_WHALE);
        IERC20(WETH).transfer(address(executor), 1 ether);
        IERC20(WETH).transfer(alice, 1000 ether);
        assertEq(IERC20(WETH).balanceOf(address(executor)), 1 ether);
        vm.stopPrank();

        // 10000 USDC tokens
        vm.startPrank(USDC_WHALE);
        IERC20(USDC).transfer(address(executor), 1000 * (10 ** 8));
        IERC20(USDC).transfer(alice, 1500 * (10 ** 8));
        assertEq(IERC20(USDC).balanceOf(address(executor)), 1000 * (10 ** 8));
        vm.stopPrank();
    }

    function testCalcMetrics() public view {
        // Calculate the metrics for the pool
        Executor.MetricParams memory metric = executor.calcMetrics(POOL_ADDR);
        assertEq(metric.token0, USDC);
        assertEq(metric.token1, WETH);
        assertEq(metric.fee, 500);

        // Assert that ticks are correct
        // sqrtPrice: 1271487029301751360839668024426277
        assertEq(metric.tick, 193677);
        // sqrtPrice at lower tick: 1271042108952745193971131637216017
        assertEq(metric.tickLower, 193670);
        // sqrtPrice at upper tick: 1271677757124143518465928949549624
        assertEq(metric.tickUpper, 193680);
    }

    function testExecuteFailure() public {
        executor.execute(POOL_ADDR);

        // No-one swapped so benchmark should fail
        vm.expectRevert(Executor.BenchMarkFailure.selector);
        executor.finish();
    }

    function testExecuteWithSwap() public {
        executor.execute(POOL_ADDR);

        // Swap some WETH for USDC
        vm.startPrank(alice);
        IERC20(USDC).approve(address(swapRouter), 1500 * (10 ** 8));
        swapRouter.exactInputSingle(
            ISwapRouter.ExactInputSingleParams({
                tokenIn: USDC,
                tokenOut: WETH,
                fee: 500,
                recipient: address(this),
                deadline: block.timestamp,
                amountIn: 1500 * (10 ** 8),
                amountOutMinimum: 0,
                sqrtPriceLimitX96: 0
            })
        );
        vm.stopPrank();

        // Finish execution
        executor.finish();
    }
}
