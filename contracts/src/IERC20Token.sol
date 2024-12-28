// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IERC20Token {
    function transfer(address recipient, uint256 value) external returns (bool);
    function balanceOf(address recipient) external view returns (uint256 amount);
    function approve(address spender, uint256 value) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256 amount);
}
