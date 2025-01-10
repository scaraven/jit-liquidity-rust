// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {FakeUniswapV3Pool} from "@mock/FakeUniswapV3Pool.sol";

contract WhitelistTest is Test {
    Whitelist whitelist;
    address alice;
    address bob;

    address constant POOL_ADDR = address(0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640);
    address constant WETH = address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

    address constant FACTORY = address(0x1F98431c8aD98523631AE4a59f267346ea31F984);

    function setUp() public {
        vm.createSelectFork(vm.envString("INFURA_URL"), vm.envUint("INFURA_URL_BLOCK"));
        // Dummy addresses
        alice = makeAddr("alice");
        bob = makeAddr("bob");

        // Initialize the Whitelist contract
        whitelist = new Whitelist(alice, FACTORY);
    }

    function testAddToWhitelist() public {
        vm.startPrank(alice);

        // Test adding an address to the whitelist
        whitelist.addWhitelist(POOL_ADDR);

        assert(whitelist.checkWhitelist(POOL_ADDR));
        vm.stopPrank();

        // Test adding an address to the whitelist with a non-owner
        vm.startPrank(bob);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, bob));
        whitelist.addWhitelist(POOL_ADDR);
        vm.stopPrank();

        // Attempt adding a fake pool address
        vm.startPrank(alice);
        FakeUniswapV3Pool fakePool = new FakeUniswapV3Pool(WETH, USDC, 500);
        vm.expectRevert(abi.encodeWithSelector(Whitelist.UnauthorizedPool.selector, POOL_ADDR, address(fakePool)));
        whitelist.addWhitelist(address(fakePool));
        vm.stopPrank();
    }

    function testRemoveFromWhitelist() public {
        vm.startPrank(alice);
        // Test adding an address to the whitelist
        whitelist.addWhitelist(POOL_ADDR);
        vm.stopPrank();

        // Assert that a non-owner cannot remove an address from the whitelist
        vm.startPrank(bob);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, bob));
        whitelist.removeWhitelist(POOL_ADDR);
        vm.stopPrank();

        // Assert that an owner can remove an address from the whitelist
        vm.startPrank(alice);
        whitelist.removeWhitelist(POOL_ADDR);
        assert(!whitelist.checkWhitelist(POOL_ADDR));
        vm.stopPrank();
    }

    function testCheckWhitelist() public {
        // Test checking if an address is in the whitelist
        vm.startPrank(alice);
        whitelist.addWhitelist(POOL_ADDR);
        vm.stopPrank();

        bool isWhitelisted = whitelist.checkWhitelist(POOL_ADDR);
        assertEq(isWhitelisted, true);

        assert(!whitelist.checkWhitelist(makeAddr("random")));
    }
}
