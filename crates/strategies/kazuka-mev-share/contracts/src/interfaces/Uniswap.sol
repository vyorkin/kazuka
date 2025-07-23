// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

interface IUniswapV3Pool {
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

interface ISwapRouter02 {
  struct ExactInputSingleParams {
    /// Address of the ERC20 token to swap from (token to send).
    address tokenIn;
    /// Address of the ERC20 token to swap to (token to receive).
    address tokenOut;
    /// Fee tier of the Uniswap V3 pool.
    /// For example: 500 = 0.05%, 3000 = 0.3%, 10000 = 1%.
    uint24 fee;
    /// Recipient of output tokens.
    address recipient;
    /// Swap transaction deadline.
    /// Unix timestamp.
    uint256 deadline;
    /// Input token amount to swap.
    uint256 amountIn;
    /// Minmum acceptable output token amount (slippage protection).
    /// If minimum output isn’t met then entire swap is reverted.
    uint256 amountOutMinimum;
    /// Price limit for the swap.
    /// Set to 0 for no limit.
    /// Allows partial swaps stopping at the price limit.
    uint160 sqrtPriceLimitX96;
  }

  function exactInputSingle(ExactInputSingleParams calldata params)
    external payable returns (uint256 amountOut);
}

// sqrtPriceLimitX96
//
// It is a price limit parameter used during swaps on Uniswap V3.
// Represents the square root of the price.
// Encoded in Q64.96 fixed-point format:
// - 64 bits for the integer part
// - 96 bits for the fractional part
//
// It allows you to set a price boundary that the swap cannot exceed.
//
// * During a swap, the pool price changes depending on liquidity consumption.
// * sqrtPriceLimitX96 sets a limit on how far the price can move during the swap.
// * The swap will proceed only as long as the current price stays within this limit.
// * If the price has already exceeded the limit at the start, the swap will revert.
// * If you set sqrtPriceLimitX96 to zero, no price limit is applied, and the swap can proceed without restriction.
//
// What for:
// * Protection against price slippage during the swap.
// * Controlling swaps so price doesn’t move beyond a safe limit.
// * In arbitrage to manage price impact.
// * Helps prevent unfavorable execution especially in volatile or low liquidity environments.
