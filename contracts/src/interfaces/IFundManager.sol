// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFundManager {
    function startBenchmark(address client, address[] calldata tokens) external;
    function endBenchmark(address client, address[] calldata tokens) external returns (bool);
    function setOracle(address oracle) external;
}
