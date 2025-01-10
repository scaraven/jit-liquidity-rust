// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IFundManager} from "./interfaces/IFundManager.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IOracle} from "./interfaces/IOracle.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

error BenchmarkNotStarted();

event FundChange(uint256, uint256);

contract FundManager is IFundManager, Ownable {
    uint256 public constant DECIMALS = 8;
    uint256 public constant ETH_DECIMALS = 18;

    IOracle public oracle;

    uint256 public usd_value;

    constructor(address _owner) Ownable(_owner) {}

    function startBenchmark(address client, address[] calldata tokens) external override onlyOwner {
        usd_value = calculateUSDValue(client, tokens);
    }

    function endBenchmark(address client, address[] calldata tokens) external override onlyOwner returns (bool) {
        require(usd_value != 0, BenchmarkNotStarted());
        uint256 current_usd_value = calculateUSDValue(client, tokens);

        emit FundChange(usd_value, current_usd_value);

        // Ensure that our portfolio has not decreased
        return current_usd_value > usd_value;
    }

    function setOracle(address _oracle) external override onlyOwner {
        oracle = IOracle(_oracle);
    }

    function calculateUSDValue(address client, address[] calldata tokens) public view returns (uint256) {
        // Loop through all the tokens and calcualte the total USD value
        uint256 total_usd_value = 0;

        for (uint256 i = 0; i < tokens.length; i++) {
            // Ensure decimals is correct!
            uint256 price = oracle.getPrice(address(tokens[i]));

            uint256 balance;
            uint256 decimals;
            if (tokens[i] == address(0)) {
                balance = client.balance;
                decimals = ETH_DECIMALS;
            } else {
                balance = ERC20(tokens[i]).balanceOf(client);
                decimals = ERC20(tokens[i]).decimals();
            }

            decimals += oracle.getDecimals();
            if (decimals < DECIMALS) {
                total_usd_value += price * balance * (10 ** (DECIMALS - decimals));
            } else {
                total_usd_value += price * balance / (10 ** (decimals - DECIMALS));
            }
        }
        return total_usd_value;
    }
}
