// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager, Pool} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IOracle} from "./interfaces/IOracle.sol";
import {IERC20Token} from "./interfaces/IERC20Token.sol";

contract FundManager is IFundManager, Ownable {
    mapping(address => Pool) private whitelist;

    IOracle public oracle;

    uint256 public usd_value;

    constructor(address _owner) Ownable(_owner) {}

    function check_whitelist(address pool) external view override returns (bool) {
        return whitelist[pool].token0 != address(0) && whitelist[pool].token1 != address(0);
    }

    function add_whitelist(address pool, address token0, address token1) external override onlyOwner returns (bool) {
        whitelist[pool] = Pool(token0, token1);
        return true;
    }

    function get_whitelist(address pool) external view override returns (address, address) {
        return (whitelist[pool].token0, whitelist[pool].token1);
    }

    function start_benchmark(address client, IERC20Token[] calldata tokens) external override onlyOwner {
        usd_value = calculate_usd_value(client, tokens, true);
    }

    function end_benchmark(address client, IERC20Token[] calldata tokens)
        external
        view
        override
        onlyOwner
        returns (bool)
    {
        require(usd_value != 0, "FUNDMANAGER: Benchmark not started");
        uint256 current_usd_value = calculate_usd_value(client, tokens, true);

        // Ensure that our portfolio has not decreased
        return current_usd_value > usd_value;
    }

    function setOracle(IOracle _oracle) external onlyOwner {
        oracle = _oracle;
    }

    function calculate_usd_value(address client, IERC20Token[] calldata tokens, bool eth)
        internal
        view
        returns (uint256)
    {
        // Loop through all the tokens and calcualte the total USD value
        uint256 total_usd_value = 0;

        for (uint256 i = 0; i < tokens.length; i++) {
            // Ensure decimals is correct!
            uint256 price = oracle.get_price(address(tokens[i]));
            uint256 balance = tokens[i].balanceOf(client);
            total_usd_value += price * balance;
        }

        // Ensure we calculate ETH value as well
        if (eth) {
            uint256 eth_price = oracle.get_price(address(0));
            uint256 eth_balance = client.balance;
            total_usd_value += eth_price * eth_balance;
        }

        return total_usd_value;
    }
}
