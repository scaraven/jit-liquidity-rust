// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

struct Pool {
    address token0;
    address token1;
}

contract Whitelist is Ownable {
    mapping(address => Pool) private whitelist;

    constructor(address _owner) Ownable(_owner) {}

    function check_whitelist(address pool) external view returns (bool) {
        return whitelist[pool].token0 != address(0) && whitelist[pool].token1 != address(0);
    }

    function add_whitelist(address pool, address token0, address token1) external onlyOwner returns (bool) {
        whitelist[pool] = Pool(token0, token1);
        return true;
    }

    function get_whitelist(address pool) external view returns (address, address) {
        return (whitelist[pool].token0, whitelist[pool].token1);
    }
}
