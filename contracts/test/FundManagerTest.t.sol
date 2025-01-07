// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {MockOracle} from "../src/mocks/MockOracle.sol";
import {FundManager, BenchmarkNotStarted} from "../src/FundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {MockERC20} from "../src/mocks/MockERC20.sol";

contract FundManagerTest is Test {
    FundManager manager;
    MockERC20 token0;
    MockERC20 token1;

    address oracle;
    address alice;

    function setUp() public {
        oracle = address(new MockOracle());
        manager = new FundManager(address(this));
        manager.set_oracle(oracle);

        alice = makeAddr("alice");

        // Setup tokens
        token0 = new MockERC20("", "", 18, 10);
        token1 = new MockERC20("", "", 18, 100);

        // Set default prices
        MockOracle(oracle).set_price(address(token0), 100);
        MockOracle(oracle).set_price(address(token1), 5);
    }

    function testOnlyOwnerCanInteract() public {
        vm.startPrank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        manager.set_oracle(oracle);

        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        address[] memory tokens = new address[](2);
        manager.start_benchmark(alice, tokens);

        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        manager.end_benchmark(alice, tokens);
    }

    function testPortfolioIncrease() public {
        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        assertEq(manager.calculate_usd_value(address(this), tokens), 1500);

        manager.start_benchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).set_price(address(token1), 3);

        // Transfer away some token0
        token0.transfer(alice, 1);
        // Gain some token1
        token1.mint(address(this), 101);

        assert(manager.end_benchmark(address(this), tokens));

        assertEq(manager.calculate_usd_value(address(this), tokens), 1503);
    }

    function testPortfolioDecrease() public {
        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        assertEq(manager.calculate_usd_value(address(this), tokens), 1500);

        manager.start_benchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).set_price(address(token1), 1);

        // Transfer away some token0
        token0.transfer(alice, 2);
        // Gain some token1
        token1.mint(address(this), 1);

        assert(!manager.end_benchmark(address(this), tokens));

        assertEq(manager.calculate_usd_value(address(this), tokens), 901);
    }

    function testPortfolioIncreaseEth() public {
        vm.deal(address(this), 10);

        MockOracle(oracle).set_price(address(0), 1000);

        address[] memory tokens = new address[](3);
        tokens[0] = address(token0);
        tokens[1] = address(token1);
        tokens[2] = address(0);

        assertEq(manager.calculate_usd_value(address(this), tokens), 11500);

        manager.start_benchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).set_price(address(0), 2000);

        manager.end_benchmark(address(this), tokens);

        assertEq(manager.calculate_usd_value(address(this), tokens), 21500);
    }

    function testOnlyEndBenchmarkAfterStarting() public {
        vm.expectRevert(BenchmarkNotStarted.selector);
        address[] memory tokens = new address[](0);
        manager.end_benchmark(address(this), tokens);
    }
}
