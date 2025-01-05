// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IERC20Token} from "./IERC20Token.sol";

struct Pool {
    address token0;
    address token1;
}

interface IFundManager {
    function check_whitelist(address pool) external view returns (bool);
    function add_whitelist(address pool, address token0, address token1) external returns (bool);
    function get_whitelist(address pool) external view returns (address, address);

    function start_benchmark(address client, IERC20Token[] calldata tokens) external;
    function end_benchmark(address client, IERC20Token[] calldata tokens) external view returns (bool);
}
