// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IERC20Token} from "./IERC20Token.sol";

interface IFundManager {
    function start_benchmark(address client, address[] calldata tokens) external;
    function end_benchmark(address client, address[] calldata tokens) external view returns (bool);
    function set_oracle(address oracle) external;
}
