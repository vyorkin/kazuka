// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/// @title The interface for a Uniswap V3 Pool
/// @notice A Uniswap pool facilitates swapping and automated market making between any two assets that strictly conform
/// to the ERC20 specification
/// @dev The pool interface is broken up into many smaller pieces
interface IUniswapV3Pool {
    /// @notice The first of the two tokens of the pool, sorted by address
    /// @return The token contract address
    function token0() external view returns (address);

    /// @notice The second of the two tokens of the pool, sorted by address
    /// @return The token contract address
    function token1() external view returns (address);

    /// @notice The pool's fee in hundredths of a bip, i.e. 1e-6
    /// @return The fee
    function fee() external view returns (uint24);

    /// @notice Swaps token0 for token1, or vise versa
    /// @param recipient Recipient address
    /// @param zeroForOne Swap direction, true if token0 -> token1, false for token1 -> token0
    /// @param amountSpecified Swap amount, with sign:
    ///                        - positive: You specify how much token you send
    ///                        - negative: You specify how much token you want to receive
    /// @param sqrtPriceLimitX96 Q64.96 sqrt price limit that the swap cannot exceed (see below).
    //                           Can be used for slippage protection
    /// @param data Arbitrary data that will be passed to the callback function
    ///
    /// @return amount0 The change (delta) in token0 balance of the pool as a result of the swap:
    ///                 - positive: pool received token0
    ///                 - negative: pool sent token0
    /// @return amount1 The change (delta) in token1 balance of the pool as a result of the swap:
    ///                 - positive: pool received token1
    ///                 - negative: pool sent token1
    function swap(
        address recipient,
        bool zeroForOne,
        int256 amountSpecified,
        uint160 sqrtPriceLimitX96,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);
    //                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // These return values allow determining exactly
    // how much of each token the pool received and paid out during the swap.
}
