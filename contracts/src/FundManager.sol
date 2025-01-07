// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IOracle} from "./interfaces/IOracle.sol";
import {IERC20Token} from "./interfaces/IERC20Token.sol";

contract FundManager is IFundManager, Ownable {
    IOracle public oracle;

    uint256 public usd_value;

    constructor(address _owner) Ownable(_owner) {}

    function start_benchmark(address client, address[] calldata tokens) external override onlyOwner {
        usd_value = calculate_usd_value(client, tokens);
    }

    function end_benchmark(address client, address[] calldata tokens) external view override onlyOwner returns (bool) {
        require(usd_value != 0, "FUNDMANAGER: Benchmark not started");
        uint256 current_usd_value = calculate_usd_value(client, tokens);

        // Ensure that our portfolio has not decreased
        return current_usd_value > usd_value;
    }

    function setOracle(IOracle _oracle) external onlyOwner {
        oracle = _oracle;
    }

    function calculate_usd_value(address client, address[] calldata tokens) internal view returns (uint256) {
        // Loop through all the tokens and calcualte the total USD value
        uint256 total_usd_value = 0;

        for (uint256 i = 0; i < tokens.length; i++) {
            // Ensure decimals is correct!
            uint256 price = oracle.get_price(address(tokens[i]));

            uint256 balance;
            if (tokens[i] == address(0)) {
                balance = client.balance;
            } else {
                balance = IERC20Token(tokens[i]).balanceOf(client);
            }
            total_usd_value += price * balance;
        }
        return total_usd_value;
    }
}
