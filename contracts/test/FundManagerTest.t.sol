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

    MockERC20 token2;
    MockERC20 token3;

    address oracle;
    address alice;

    function setUp() public {
        oracle = address(new MockOracle());
        manager = new FundManager(address(this));
        manager.setOracle(oracle);

        alice = makeAddr("alice");

        // Setup tokens
        token0 = new MockERC20("", "", 0, 10);
        token1 = new MockERC20("", "", 0, 100);

        // Tokens with realistic decimals
        token2 = new MockERC20("", "", 8, 10 * 10 ** 8);
        token3 = new MockERC20("", "", 18, 1 ether);

        // Set default prices
        MockOracle(oracle).setPrice(address(token0), 100);
        MockOracle(oracle).setPrice(address(token1), 5);

        // Set decimals
        MockOracle(oracle).setPrice(address(token2), 2 * 10 ** 8);
        MockOracle(oracle).setPrice(address(token3), 5 * 10 ** 8);
    }

    function testOnlyOwnerCanInteract() public {
        vm.startPrank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        manager.setOracle(oracle);

        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        address[] memory tokens = new address[](2);
        manager.startBenchmark(alice, tokens);

        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        manager.endBenchmark(alice, tokens);
    }

    function testPortfolioIncrease() public {
        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        assertEq(manager.calculateUSDValue(address(this), tokens), 1500);

        manager.startBenchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).setPrice(address(token1), 3);

        // Transfer away some token0
        token0.transfer(alice, 1);
        // Gain some token1
        token1.mint(address(this), 101);

        assert(manager.endBenchmark(address(this), tokens));

        assertEq(manager.calculateUSDValue(address(this), tokens), 1503);
    }

    function testPortfolioDecrease() public {
        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        assertEq(manager.calculateUSDValue(address(this), tokens), 1500);

        manager.startBenchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).setPrice(address(token1), 1);

        // Transfer away some token0
        token0.transfer(alice, 2);
        // Gain some token1
        token1.mint(address(this), 1);

        assert(!manager.endBenchmark(address(this), tokens));

        assertEq(manager.calculateUSDValue(address(this), tokens), 901);
    }

    function testPortfolioIncreaseEth() public {
        vm.deal(address(this), 1 ether);

        MockOracle(oracle).setPrice(address(0), 10000);

        address[] memory tokens = new address[](3);
        tokens[0] = address(token0);
        tokens[1] = address(token1);
        tokens[2] = address(0);

        assertEq(manager.calculateUSDValue(address(this), tokens), 11500);

        manager.startBenchmark(address(this), tokens);

        // Increase value of portfolio
        MockOracle(oracle).setPrice(address(0), 20000);

        manager.endBenchmark(address(this), tokens);

        assertEq(manager.calculateUSDValue(address(this), tokens), 21500);
    }

    function testOnlyEndBenchmarkAfterStarting() public {
        vm.expectRevert(BenchmarkNotStarted.selector);
        address[] memory tokens = new address[](0);
        manager.endBenchmark(address(this), tokens);
    }

    function testBenchmarkWithDifferentDecimals() public view {
        address[] memory tokens = new address[](2);
        tokens[0] = address(token2);
        tokens[1] = address(token3);

        assertEq(manager.calculateUSDValue(address(this), tokens), 25 * (10 ** manager.DECIMALS()));
    }
}
