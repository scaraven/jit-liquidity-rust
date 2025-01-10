// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IOracle} from "../interfaces/IOracle.sol";

contract MockOracle is IOracle {
    mapping(address => uint256) prices;

    function setPrice(address token, uint256 _price) external {
        prices[token] = _price;
    }

    function getPrice(address token) external view override returns (uint256) {
        return prices[token];
    }

    function getDecimals() external pure override returns (uint256) {
        return 8;
    }
}
