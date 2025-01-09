// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IUniswapV3Factory} from "@uniswap/v3-core/contracts/interfaces/IUniswapV3Factory.sol";
import {IUniswapV3Pool} from "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";

contract Whitelist is Ownable {
    mapping(address => bool) private whitelist;
    IUniswapV3Factory public immutable uniswapFactory;

    error UnauthorizedPool(address expected, address actual);

    constructor(address _owner, address _factory) Ownable(_owner) {
        uniswapFactory = IUniswapV3Factory(_factory);
    }

    function checkWhitelist(address pool) external view returns (bool) {
        return whitelist[pool];
    }

    function addWhitelist(address pool) external onlyOwner returns (bool) {
        address token0 = IUniswapV3Pool(pool).token0();
        address token1 = IUniswapV3Pool(pool).token1();

        uint24 fee = IUniswapV3Pool(pool).fee();

        // Fetch address of the pool from the factory
        address poolAddress = uniswapFactory.getPool(token0, token1, fee);
        require(poolAddress == pool, UnauthorizedPool(poolAddress, pool));

        whitelist[pool] = true;
        return true;
    }

    function removeWhitelist(address pool) external onlyOwner returns (bool) {
        delete whitelist[pool];
        return true;
    }
}
