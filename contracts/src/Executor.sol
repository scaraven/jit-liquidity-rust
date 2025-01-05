// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager, Pool} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

// Is a proxy contract that allows the owner to execute sandwich trades
contract Executor {
    IFundManager public fundManager;

    constructor(address _fundManager) {
        fundManager = IFundManager(_fundManager);
    }

    function execute(address pool, address token0, address token1) external {
        fundManager.add_whitelist(pool, token0, token1);
    }
}
