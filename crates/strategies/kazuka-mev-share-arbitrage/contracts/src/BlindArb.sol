// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {console} from "forge-std/console.sol";

import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {Owned} from "solmate/auth/Owned.sol";

import {IWETH} from "./interfaces/tokens/IWETH.sol";
import {IUniswapV3SwapCallback} from "./interfaces/IUniswapV3SwapCallback.sol";
import {ISwapRouter} from "./interfaces/ISwapRouter.sol";
import {IUniswapV3Pool} from "./interfaces/IUniswapV3Pool.sol";
import {IUniswapV2Pair} from "./interfaces/IUniswapV2Pair.sol";

contract BlindArb is Owned, IUniswapV3SwapCallback {
    IWETH internal constant weth = IWETH(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    IERC20 internal constant plu = IERC20(0xD8912C10681D8B21Fd3742244f44658dBA12264E);

    // Needed to calculate the price limit.
    // These values are taken from the Uniswap core contracts.
    uint160 private constant MIN_SQRT_RATIO = 4295128739;
    uint160 private constant MAX_SQRT_RATIO = 1461446703485210103287273052203988822378723970342;

    constructor() Owned(msg.sender) {}

    // @notice Executes arbitrage WETH -> PLU -> WETH, where
    //         token0 = PLU
    //         token1 = WETH
    function execute_weth_token1(address v2Pool, address v3Pool, uint256 amountIn, uint256 percentageToPayToCoinbase)
        public
        onlyOwner
    {
        uint256 wethBalanceBefore = weth.balanceOf(address(this));

        uint160 sqrtPriceLimitX96 = MAX_SQRT_RATIO - 1;
        address beneficiary = msg.sender;
        bytes memory callbackData = abi.encode(
            beneficiary, // msg.sender
            v2Pool, // WETH/PLU
            v3Pool, // WETH/PLU
            address(weth), // tokenIn = WETH
            address(plu), // tokenOut = PLU
            amountIn, // 1 ether (WETH)
            false // zerForOne = false (token0 <- token1 = PLU <- WETH)
        );
        (int256 pluAmountDelta, int256 _wethAmountDelta) = IUniswapV3Pool(v3Pool).swap(
            v2Pool, // recipient (UniswapV2 Pool)
            false, // zeroForOne (trading direction: token0 <- token1 = PLU <- WETH)
            int256(amountIn), // amountSpecified (amount paid to UniswapV3 pool)
            sqrtPriceLimitX96, // sqrtPriceLimitX96
            callbackData // data forwarded to uniswapV3SwapCallback
        );

        uint256 pluAmountIn = uint256(-pluAmountDelta);

        (uint256 pluReserveV2, uint256 wethReserveV2,) = IUniswapV2Pair(v2Pool).getReserves();
        //        ^^ token0             ^^ token1

        // Calculate the WETH amount we'll get after second swap PLU -> WETH on UniswapV2 pool
        uint256 wethAmountOut = uniswapV2CalculateAmountOut(
            pluAmountIn, // amountIn
            pluReserveV2, // reserveIn
            wethReserveV2 // reserveOut
        );

        IUniswapV2Pair(v2Pool).swap(
            0, // amount0Out (token0 = PLU)
            wethAmountOut, // amount1Out (token1 = WETH)
            address(this), // receiver
            "" // data
        );

        uint256 wethBalanceAfter = weth.balanceOf(address(this));

        int256 profit = int256(wethBalanceAfter) - int256(wethBalanceBefore);
        require(profit > 0, "no profit");

        uint256 profitToCoinbase = uint256(profit) * percentageToPayToCoinbase / 100;

        weth.withdraw(profitToCoinbase);
        block.coinbase.transfer(profitToCoinbase);
        require(wethBalanceAfter - profitToCoinbase > wethBalanceBefore, "arbitrage failed");
    }

    // @notice Executes arbitrage WETH -> PLU -> WETH, where
    //         token0 = WETH
    //         token1 = PLU
    // @param v2Pool UniswapV2 pool address
    // @param v3Pool UniswapV3 pool address
    // @param amountIn Amount of WETH to run arbitrage with
    function execute_weth_token0(address v2Pool, address v3Pool, uint256 amountIn, uint256 percentageToPayToCoinbase)
        public
        onlyOwner
    {
        uint256 wethBalanceBefore = weth.balanceOf(address(this));

        // Limit of the price that we're going to accept.
        // Must be: MIN_SQRT_RATIO < price_limit < MAX_SQRT_RATIO.
        // We want to set it to MIN + 1 or MAX - 1 (depending on trade direction).
        //
        // What will happen during trading:
        // * token0 -> token1 (zeroForOne = true):  sqrt price will decrease
        // * token1 -> token0 (zeroForOne = false): sqrt price will increase
        //
        // Hence:
        // * MIN_SQRT_RATIO + 1 - the minimum that the price can decrease to
        // * MAX_SQRT_RATIO - 1 - the maximum that the price can reach to
        //
        // Since we're trading only in one direction (WETH -> PLU) it is a constant
        uint160 sqrtPriceLimitX96 = MIN_SQRT_RATIO + 1;

        // Swap direction, true if token0 -> token1, false for token1 -> token0
        // The UniswapV3Pool contract has these two fields and in our case
        //   token0 = WETH
        //   token1 = PLU
        // You can go to
        // https://etherscan.io/address/0xe11ee9c18d03b43d6a7fc53e51aedda8451e837a#readContract (UniswapV3Pool WETH/PLU)
        // https://etherscan.io/address/0x87c9524237a19338be7dbcac01d6d208ff31136f#readContract (UniswapV2Pair WETH/PLU)
        // and check values of token0 and token1
        // bool zeroForOne = true; // WETH -> PLU

        address beneficiary = msg.sender;

        // We encode:
        // 1. msg.sender (caller of this contract)
        //    In uniswapV3SwapCallback want to know the beneficiary -
        //    the initial caller (msg.sender) -- who initiated the arbitrage
        // 2. Other values (a few of them aren't needed at this point)
        bytes memory callbackData = abi.encode(
            beneficiary, // msg.sender
            v2Pool, // WETH/PLU
            v3Pool, // WETH/PLU
            address(weth), // tokenIn = WETH
            address(plu), // tokenOut = PLU
            amountIn, // 1 ether (WETH)
            true // zeroForOne = true (token0 -> token1 = WETH -> PLU)
        );

        console.log("IUniswapV3Pool(v3Pool).swap [WETH -> PLU]:");
        console.log("  v2Pool = %s", v2Pool);
        console.log("  zeroForOne = %s", true);
        console.log("  amountIn = %s", amountIn);
        console.log("  sqrtPriceLimitX96 = %s", sqrtPriceLimitX96);

        // Swap on UniswapV3: WETH -> PLU
        // It will callback the uniswapV3SwapCallback and forward the
        // encoded data we provided here as last argument
        (int256 wethAmountDelta, int256 pluAmountDelta) = IUniswapV3Pool(v3Pool).swap(
            v2Pool, // recipient (UniswapV2 Pool)
            true, // zeroForOne (trading direction: WETH -> PLU)
            int256(amountIn), // amountSpecified (amount paid to UniswapV3 pool)
            sqrtPriceLimitX96, // sqrtPriceLimitX96
            callbackData // data forwarded to uniswapV3SwapCallback
        );
        // wethAmountDelta, pluAmountDelta --
        //   how much of each token the pool received and paid out during the swap

        console.log("UniswapV3 pool WETH/PLU changes:");
        console.log("  wethAmountDelta (amount0) = %s wei (WETH)", wethAmountDelta);
        console.log("  pluAmountDelta (amount1)  = %s wei (PLU)", pluAmountDelta);

        // Get WETH, PLU reserves in UniswapV2 pool to caculate the PLU amount we can get
        (uint256 wethReserveV2, uint256 pluReserveV2,) = IUniswapV2Pair(v2Pool).getReserves();
        //       ^-- WETH (reserve0)    ^-- PLU (reserve1)
        //             ^                     ^
        //          token0                 token1

        console.log("IUniswapV2Pair(v2Pool).getReserves()");
        console.log("  wethReserveV2 = %s wei (WETH)", wethReserveV2);
        console.log("  pluReserveV2  = %s wei (PLU)", pluReserveV2);

        // PLU amount we got after first swap WETH -> PLU on UniswapV3 pool
        uint256 pluAmountIn = uint256(-pluAmountDelta);
        // Calculate the WETH amount we'll get after second swap PLU -> WETH on UniswapV2 pool
        uint256 wethAmountOut = uniswapV2CalculateAmountOut(
            pluAmountIn, // amountIn
            pluReserveV2, // reserveIn
            wethReserveV2 // reserveOut
        );

        console.log("uniswapV2CalculateAmountOut(pluAmountIn, pluReserveV2, wethReserveV2)");
        console.log("  pluAmountIn   = %s wei (PLU)", pluAmountIn);
        console.log("  wethAmountOut = %s wei (WETH)", wethAmountOut);

        console.log("IUniswapV2Pair(v2Pool).swap()");

        // Swap PLU -> WETH on UniswapV2
        //
        // The swap() function takes two output amounts, one for each token
        // These are the amounts that caller wants to get in exchange for their tokens
        IUniswapV2Pair(v2Pool).swap(
            wethAmountOut, // amount0Out (token0 = WETH)
            0, // amount1Out (token1 = PLU)
            address(this), // receiver
            "" // data
        );

        uint256 wethBalanceAfter = weth.balanceOf(address(this));

        int256 profit = int256(wethBalanceAfter) - int256(wethBalanceBefore);
        require(profit > 0, "no profit");

        // TODO: Add comment
        uint256 profitToCoinbase = uint256(profit) * percentageToPayToCoinbase / 100;

        weth.withdraw(profitToCoinbase);
        block.coinbase.transfer(profitToCoinbase);

        console.log("  wethBalanceBefore = %s wei (WETH)", wethBalanceBefore);
        console.log("  wethBalanceAfter = %s wei (WETH)", wethBalanceAfter);
        console.log("  profitToCoinbase = %s wei (WETH)", profitToCoinbase);
        console.log("  profit = %s wei (WETH)", profit);

        require(wethBalanceAfter - profitToCoinbase > wethBalanceBefore, "arbitrage failed");
    }

    // TODO: Extract to helper library
    function uniswapV2CalculateAmountOut(uint256 amountIn, uint256 reserveIn, uint256 reserveOut)
        private
        pure
        returns (uint256 amountOut)
    {
        uint256 amountInWithFee = amountIn * 997; // 0.3% fee
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = reserveIn * 1000 + amountInWithFee;
        amountOut = numerator / denominator;
    }

    /// @notice Uniswap V3 callback for swaps
    /// @param amount0Delta Swapped amount of token0 with sign (WETH)
    /// @param amount1Delta Swapped amount of token1 with sign (PLU)
    /// @param data Encoded data from the last argument to IUniswapV3Pool.swap that we called
    function uniswapV3SwapCallback(int256 amount0Delta, int256 amount1Delta, bytes calldata data) external {
        console.log("uniswapV3SwapCallback()");
        console.log("  amount0Delta = %s", amount0Delta);
        console.log("  amount1Delta = %s", amount1Delta);

        // amount0Delta - The amount of token0 that was sent (negative) or
        //                must be received (positive) by the pool by the end of the swap
        //                If positive, the callback must send that amount of token0 to the pool
        // amount1Delta - The amount of token1 that was sent (negative) or
        //                must be received (positive) by the pool by the end of the swap
        //                If positive, the callback must send that amount of token1 to the pool

        // amount0Delta and amount1Delta can both be 0 if no tokens were swapped
        require((amount0Delta > 0 && amount1Delta < 0) || (amount0Delta < 0 && amount1Delta > 0), "nothing to repay");

        // amount0Delta = 1000000000000000000
        // amount1Delta = -247772059380214365559

        // We got our PLU, now we want to pay back the WETH to the UniswapV3 pool

        // Decode the callback data
        (
            address beneficiary,
            address v2Pool, // WETH/PLU
            address v3Pool, // WETH/PLU
            address tokenIn, // WETH
            address tokenOut, // PLU
            uint256 amountIn, // Amount that we have to pay back to UniswapV3 pool
            bool zeroForOne
        ) = abi.decode(data, (address, address, address, address, address, uint256, bool));
        // Only UniswapV3 pool is allowed to call this function
        require(msg.sender == v3Pool, "invalid sender");

        console.log("Decoded callback data:");
        console.log("  beneficiary = %s", beneficiary);
        console.log("  v2Pool = %s", v2Pool);
        console.log("  v3Pool = %s", v3Pool);
        console.log("  tokenIn = %s", tokenIn);
        console.log("  tokenOut = %s", tokenOut);
        console.log("  amountIn = %s", amountIn);
        console.log("  zeroForOne = %s", zeroForOne);

        (address tokenToRepay, uint256 amountToRepay) = amount0Delta > 0
            ? (IUniswapV3Pool(v3Pool).token0(), uint256(amount0Delta))
            : (IUniswapV3Pool(v3Pool).token1(), uint256(amount1Delta));

        console.log("amountToRepay = %s wei (WETH)", amountToRepay);

        // Repay WETH for UniswapV3 pool
        IERC20(tokenToRepay).transfer(v3Pool, amountToRepay);
        // ^ Equaivalent to: weth.transfer(v3Pool, amountToRepay);
    }

    function withdrawWETH() external onlyOwner {
        uint256 balance = weth.balanceOf(address(this));
        weth.transfer(msg.sender, balance);
    }

    function withdrawETH() external onlyOwner {
        uint256 balance = address(this).balance;
        payable(msg.sender).transfer(balance);
    }

    receive() external payable {}
}
