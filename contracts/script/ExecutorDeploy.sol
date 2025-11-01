// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";

import {Executor} from "../src/Executor.sol";
import {FundManager} from "../src/FundManager.sol";
import {Whitelist} from "../src/Whitelist.sol";
import {Oracle} from "../src/Oracle.sol";

contract ExecutorDeploy is Script {
    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        require(deployerPrivateKey != 0, "PRIVATE_KEY not set");

        vm.startBroadcast(deployerPrivateKey);

        address deployerAddress = vm.addr(deployerPrivateKey);

        FundManager manager = new FundManager(deployerAddress);
        Whitelist list = new Whitelist(deployerAddress, 0x1F98431c8aD98523631AE4a59f267346ea31F984);
        Oracle oracle = new Oracle();

        Executor executor = new Executor(deployerAddress, address(manager), address(list));

        manager.setOracle(address(oracle));

        vm.stopBroadcast();
    }
}
