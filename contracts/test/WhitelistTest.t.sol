// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract WhitelistTest is Test {
    Whitelist whitelist;
    address alice;
    address bob;

    address constant POOL_ADDR = address(0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc);
    address constant WETH = address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

    function setUp() public {
        // Dummy addresses
        alice = makeAddr("alice");
        bob = makeAddr("bob");

        // Initialize the Whitelist contract
        whitelist = new Whitelist(alice);
    }

    function testAddToWhitelist() public {
        vm.startPrank(alice);

        // Test adding an address to the whitelist
        whitelist.add_whitelist(POOL_ADDR, WETH, USDC);

        (address token0, address token1) = whitelist.get_whitelist(POOL_ADDR);
        assertEq(token0, WETH);
        assertEq(token1, USDC);
        vm.stopPrank();

        // Test adding an address to the whitelist with a non-owner
        vm.startPrank(bob);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, bob));
        whitelist.add_whitelist(POOL_ADDR, WETH, USDC);
        vm.stopPrank();
    }

    function testRemoveFromWhitelist() public {
        vm.startPrank(alice);
        // Test adding an address to the whitelist
        whitelist.add_whitelist(POOL_ADDR, WETH, USDC);
        vm.stopPrank();

        // Assert that a non-owner cannot remove an address from the whitelist
        vm.startPrank(bob);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, bob));
        whitelist.remove_whitelist(POOL_ADDR);
        vm.stopPrank();

        // Assert that an owner can remove an address from the whitelist
        vm.startPrank(alice);
        whitelist.remove_whitelist(POOL_ADDR);
        (address token0, address token1) = whitelist.get_whitelist(POOL_ADDR);
        assertEq(token0, address(0));
        assertEq(token1, address(0));
        vm.stopPrank();
    }

    function testCheckWhitelist() public {
        // Test checking if an address is in the whitelist

        vm.startPrank(alice);
        whitelist.add_whitelist(POOL_ADDR, WETH, USDC);
        vm.stopPrank();

        bool isWhitelisted = whitelist.check_whitelist(POOL_ADDR);
        assertEq(isWhitelisted, true);

        isWhitelisted = whitelist.check_whitelist(makeAddr("random"));
        assertNotEq(isWhitelisted, true);
    }
}
