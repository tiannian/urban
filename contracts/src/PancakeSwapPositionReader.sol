// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity >=0.7.5;

import "@pancakeswap/v3-core/contracts/interfaces/IPancakeV3Factory.sol";
import "@pancakeswap/v3-core/contracts/interfaces/pool/IPancakeV3PoolState.sol";
import "@pancakeswap/v3-core/contracts/libraries/TickMath.sol";
import "@pancakeswap/v3-periphery/contracts/interfaces/INonfungiblePositionManager.sol";
import "@pancakeswap/v3-periphery/contracts/libraries/LiquidityAmounts.sol";

/// @title PancakeSwap Position Reader
/// @notice Reads and normalizes LP position information from PancakeSwap V3 positions
/// @dev Implements the readPositionInfo logic as specified in 0102-read-contract.md
contract PancakeSwapPositionReader {
    /// @notice Configuration for the position reader
    struct Config {
        address positionManager;
        address factory;
        address baseToken;
        address usdtToken;
    }

    Config public immutable config;

    /// @notice Position information normalized to BASE/USDT convention
    struct PositionInfo {
        address baseToken;
        address usdtToken;
        int24 tickLower;
        int24 tickUpper;
        int24 tickCurrent;
        uint160 sqrtPriceX96;
        uint256 amount0;
        uint256 amount1;
        uint256 feesBase;
        uint256 feesUsdt;
    }

    /// @notice Error thrown when position does not exist
    error InvalidPosition();
    /// @notice Error thrown when pool does not exist
    error PoolNotFound();
    /// @notice Error thrown when pool is locked
    error PoolLocked();
    /// @notice Error thrown when pair does not match configured base/usdt tokens
    error UnsupportedPair();

    /// @notice Constructs the position reader with configuration
    /// @param _positionManager Address of the PancakeSwap V3 NonfungiblePositionManager
    /// @param _factory Address of the PancakeSwap V3 Factory
    /// @param _baseToken Address of the BASE token (e.g., BNB)
    /// @param _usdtToken Address of the USDT token
    constructor(
        address _positionManager,
        address _factory,
        address _baseToken,
        address _usdtToken
    ) {
        require(_positionManager != address(0), "Invalid position manager");
        require(_factory != address(0), "Invalid factory");
        require(_baseToken != address(0), "Invalid base token");
        require(_usdtToken != address(0), "Invalid USDT token");
        require(_baseToken != _usdtToken, "Tokens must differ");

        config = Config({
            positionManager: _positionManager,
            factory: _factory,
            baseToken: _baseToken,
            usdtToken: _usdtToken
        });
    }

    /// @notice Reads position information for a given token ID
    /// @dev Follows the canonical readPositionInfo logic from 0102-read-contract.md
    /// @param tokenId The LP position identifier (NFT id)
    /// @return info Normalized position information
    function readPositionInfo(
        uint256 tokenId
    ) external view returns (PositionInfo memory info) {
        // Step 1: Read raw position data
        (
            uint96 nonce,
            address operator,
            address token0,
            address token1,
            uint24 fee,
            int24 tickLower,
            int24 tickUpper,
            uint128 liquidity,
            uint256 feeGrowthInside0LastX128,
            uint256 feeGrowthInside1LastX128,
            uint128 tokensOwed0,
            uint128 tokensOwed1
        ) = INonfungiblePositionManager(config.positionManager).positions(
                tokenId
            );

        // Validate that position exists (nonce check is a proxy for existence)
        // Note: The actual implementation may revert on invalid tokenId, which is acceptable
        // We rely on the external call to handle this

        // Step 2: Resolve pool and read slot0
        address poolAddress = IPancakeV3Factory(config.factory).getPool(
            token0,
            token1,
            fee
        );
        if (poolAddress == address(0)) {
            revert PoolNotFound();
        }

        IPancakeV3PoolState pool = IPancakeV3PoolState(poolAddress);
        (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint32 feeProtocol,
            bool unlocked
        ) = pool.slot0();

        if (!unlocked) {
            revert PoolLocked();
        }

        // Step 3: Compute boundary prices
        uint160 sqrtRatioAX96 = TickMath.getSqrtRatioAtTick(tickLower);
        uint160 sqrtRatioBX96 = TickMath.getSqrtRatioAtTick(tickUpper);

        // Step 4: Compute raw token amounts from liquidity
        uint256 amount0Raw;
        uint256 amount1Raw;

        if (liquidity == 0) {
            // Position is empty, amounts are zero
            amount0Raw = 0;
            amount1Raw = 0;
        } else {
            (amount0Raw, amount1Raw) = LiquidityAmounts.getAmountsForLiquidity(
                sqrtPriceX96,
                sqrtRatioAX96,
                sqrtRatioBX96,
                liquidity
            );
        }

        // Step 5: Normalize to BASE and USDT
        uint256 amountBase;
        uint256 amountUsdt;
        uint256 feesBase;
        uint256 feesUsdt;

        if (token0 == config.baseToken && token1 == config.usdtToken) {
            // Case A: token0 == base_token and token1 == usdt_token
            amountBase = amount0Raw;
            amountUsdt = amount1Raw;
            feesBase = tokensOwed0;
            feesUsdt = tokensOwed1;
        } else if (token1 == config.baseToken && token0 == config.usdtToken) {
            // Case B: token1 == base_token and token0 == usdt_token
            amountBase = amount1Raw;
            amountUsdt = amount0Raw;
            feesBase = tokensOwed1;
            feesUsdt = tokensOwed0;
        } else {
            // Case C: unsupported pair
            revert UnsupportedPair();
        }

        // Step 6: Construct final output
        info = PositionInfo({
            baseToken: config.baseToken,
            usdtToken: config.usdtToken,
            tickLower: tickLower,
            tickUpper: tickUpper,
            tickCurrent: tick,
            sqrtPriceX96: sqrtPriceX96,
            amount0: amountBase,
            amount1: amountUsdt,
            feesBase: feesBase,
            feesUsdt: feesUsdt
        });
    }
}
