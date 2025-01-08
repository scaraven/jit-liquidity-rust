// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {IFundManager} from "@interfaces/IFundManager.sol";
import {MetricParams, Executor} from "@core/Executor.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract ExecutorTest is Test {
    Whitelist whitelist;
    IFundManager fundManager;
    Executor executor;

    address constant POOL_ADDR = address(0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640);
    address constant WETH = address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);
    address constant NON_FUNGIBLE_MANAGER = address(0xC36442b4a4522E871399CD717aBDD847Ab11FE88);

    address constant WETH_WHALE = address(0xF04a5cC80B1E94C69B48f5ee68a08CD2F09A7c3E);
    address constant USDC_WHALE = address(0x37305B1cD40574E4C5Ce33f8e8306Be057fD7341);

    function setUp() public {
        // Fork chain
        uint256 forkId = vm.createFork(vm.envString("INFURA_URL"), vm.envUint("INFURA_URL_BLOCK"));
        vm.selectFork(forkId);

        // Initialize the Whitelist contract
        whitelist = new Whitelist(address(this));
        fundManager = IFundManager(address(this));
        executor = new Executor(address(this), address(fundManager), address(whitelist), NON_FUNGIBLE_MANAGER);

        // Transfer some WETH and ETH to the executor
        IERC20(WETH).balanceOf(WETH_WHALE);
        IERC20(USDC).balanceOf(USDC_WHALE);

        vm.prank(WETH_WHALE);
        IERC20(WETH).transfer(address(executor), 100_000_000);

        vm.prank(USDC_WHALE);
        IERC20(USDC).transfer(address(executor), 100_000_000);
    }

    function testCalcMetrics() public {
        // Calculate the metrics for the pool
        MetricParams memory metric = executor.calc_metrics(POOL_ADDR);
        assertEq(metric.token0, USDC);
        assertEq(metric.token1, WETH);
        assertEq(metric.fee, 500);

        // Assert that ticks are correct
        assertEq(metric.tickLower, 193670);
        assertEq(metric.tickUpper, 193680);

        // Assert that amounts are correct
        assertEq(metric.amount0, 100_000_000);
        assertEq(metric.amount1, 100_000_000);
    }
}
