// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IOracle} from "../interfaces/IOracle.sol";

contract MockOracle is IOracle {
    mapping(address => uint256) prices;

    function set_price(address token, uint256 _price) external {
        prices[token] = _price;
    }

    function get_price(address token) external view override returns (uint256) {
        return prices[token];
    }
}
