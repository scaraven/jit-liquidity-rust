// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";
import {Whitelist} from "@core/Whitelist.sol";
import {IFundManager} from "@interfaces/IFundManager.sol";
import {Executor} from "@core/Executor.sol";

contract ExecutorTest is Test {
    Whitelist whitelist;
    IFundManager fundManager;
    Executor executor;

    address constant POOL_ADDR = address(0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc);
    address constant WETH = address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant USDC = address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

    function setUp() public {
        // Fork chain
        uint256 forkId = vm.createFork(vm.envString("INFURA_URL"), vm.envUint("INFURA_URL_BLOCK"));
        vm.selectFork(forkId);

        // Initialize the Whitelist contract
        whitelist = new Whitelist(address(this));
    }
}
