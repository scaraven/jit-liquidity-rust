// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IExecutor {
    function execute(address pool) external;
    function finish() external;
    function setup(address[] calldata tokens) external;
    function withdraw(address[] calldata tokens) external;

    function setFundManager(address _fundManager) external;
}
