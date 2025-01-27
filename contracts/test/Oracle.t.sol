// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";

import {Oracle} from "@core/Oracle.sol";

contract OracleTest is Test {
    Oracle public oracle;

    address constant BTC_USD = address(0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c);
    address constant ETH_USD = address(0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419);
    address constant WBTC = address(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

    function setUp() public {
        vm.createSelectFork(vm.envString("RPC_URL"), vm.envUint("RPC_TEST_URL_BLOCK"));
        oracle = new Oracle();
        oracle.setFeed(WBTC, BTC_USD);
        oracle.setFeed(address(0), ETH_USD);
    }

    function testGetPrice() public view {
        assertEq(oracle.getPrice(WBTC), 10479693812896);
        assertEq(oracle.getPrice(address(0)), 387337000000);
    }

    // TODO: Add tests for invalid and stale feeds
}
