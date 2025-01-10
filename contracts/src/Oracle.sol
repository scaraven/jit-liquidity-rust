// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IOracle} from "./interfaces/IOracle.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {AggregatorV3Interface} from "@chainlink/contracts/src/v0.8/shared/interfaces/AggregatorV3Interface.sol";

contract Oracle is IOracle, Ownable {
    mapping(address => address) feeds;

    uint256 constant VALID_TIME_PERIOD = 1 days;
    uint256 constant DECIMALS = 8;

    enum ErrorReason {
        NegativePrice,
        StaleRound,
        RoundNotFinished,
        PriceTooOld
    }

    error OracleNotSet();
    error InvalidPrice(ErrorReason reason);

    constructor() Ownable(msg.sender) {}

    // Return price normalised to 8 decimals
    function getPrice(address token) external view override returns (uint256) {
        // Obtain oracle for token
        require(feeds[token] != address(0), OracleNotSet());

        // Get price from oracle
        AggregatorV3Interface feed = AggregatorV3Interface(feeds[token]);
        (uint80 quoteRoundID, int256 quoteAnswer,, uint256 quoteTimestamp, uint80 quoteAnsweredInRound) =
            feed.latestRoundData();

        require(quoteAnswer > 0, InvalidPrice(ErrorReason.NegativePrice));
        require(quoteAnsweredInRound >= quoteRoundID, InvalidPrice(ErrorReason.StaleRound));
        require(quoteTimestamp != 0, InvalidPrice(ErrorReason.RoundNotFinished));
        require(block.timestamp - quoteTimestamp <= VALID_TIME_PERIOD, InvalidPrice(ErrorReason.PriceTooOld));

        uint8 decimals = feed.decimals();
        if (decimals != DECIMALS) {
            if (decimals < DECIMALS) {
                return uint256(quoteAnswer) * (10 ** (DECIMALS - decimals));
            } else {
                return uint256(quoteAnswer) / (10 ** (DECIMALS - decimals));
            }
        } else {
            return uint256(quoteAnswer);
        }
    }

    // Set oracle for token/USD price
    function setFeed(address token, address oracle) external onlyOwner {
        feeds[token] = oracle;
    }

    // Return oracle decimals
    function getDecimals() external pure returns (uint256) {
        return DECIMALS;
    }
}
