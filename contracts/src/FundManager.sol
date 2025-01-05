// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager, Pool} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IOracle} from "./interfaces/IOracle.sol";

contract FundManager is IFundManager, Ownable {
    mapping(address => Pool) private whitelist;

    IOracle public oracle;

    uint256 public usd_value;

    constructor(address _owner) Ownable(_owner) {}

    function check_whitelist(address pool) external view override returns (bool) {
        return whitelist[pool].token0 != address(0) && whitelist[pool].token1 != address(0);
    }

    function add_whitelist(address pool, address token0, address token1) external override returns (bool) {
        whitelist[pool] = Pool(token0, token1);
        return true;
    }

    function get_whitelist(address pool) external view override returns (address, address) {
        return (whitelist[pool].token0, whitelist[pool].token1);
    }

    function start_benchmark(address client) external override {
        // Loop through all the pools in the whitelist and calculate USD value
    }

    function end_benchmark(address client) external view override returns (bool) {
        return true;
    }
}
