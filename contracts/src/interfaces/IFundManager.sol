// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IERC20Token} from "./IERC20Token.sol";

interface IFundManager {
    function startBenchmark(address client, address[] calldata tokens) external;
    function endBenchmark(address client, address[] calldata tokens) external view returns (bool);
    function setOracle(address oracle) external;
}
