// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

import {IWETH} from "../src/interfaces/tokens/IWETH.sol";
import {IUniswapV3Pool} from "../src/interfaces/IUniswapV3Pool.sol";
import {IUniswapV2Pair} from "../src/interfaces/IUniswapV2Pair.sol";
import {ISwapRouter} from "../src/interfaces/ISwapRouter.sol";

import {BlindArb} from "../src/BlindArb.sol";

contract BlindArbTest is Test {
  uint256 internal mainnetFork;

  IWETH internal constant weth = IWETH(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
  IERC20 internal constant plu = IERC20(0xD8912C10681D8B21Fd3742244f44658dBA12264E);

  ISwapRouter internal constant uniV3Router = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);

  // WETH/PLU pool on UniswapV3
  IUniswapV3Pool internal constant uniV3Pool = IUniswapV3Pool(0xe11Ee9c18d03B43d6A7fC53e51AeDda8451e837A);
  // WETH/PLU pool on UniswapV2
  IUniswapV2Pair internal constant uniV2Pair = IUniswapV2Pair(0x87C9524237a19338be7DbCAc01D6D208fF31136F);

  // Amount of PLU required to create arbitrage opportunity
  uint256 internal constant PLU_SWAP_AMOUNT = 5000 ether;
  // Amount of WETH to run arbitrage with
  uint256 internal constant WETH_ARB_AMOUNT = 1 ether;

  uint24 internal constant FEE = 3000;

  BlindArb internal blindArb;

  function setUp() public {
    mainnetFork = vm.createFork(
      vm.envString("ALCHEMY_ETHEREUM_MAINNET_RPC_URL"),
      vm.envUint("FORK_BLOCK_NUMBER")
    );
    vm.selectFork(mainnetFork);

    blindArb = new BlindArb();

    // Get some initial WETH for the BlindArb contract
    deal(address(weth), address(blindArb), WETH_ARB_AMOUNT);
    // Get some PLU for the test contract
    deal(address(plu), address(this), PLU_SWAP_AMOUNT);
  }

  function test_arbitrage_weth_token0() public {
    vm.selectFork(mainnetFork);

    uint256 blindArbBalanceInitial = weth.balanceOf(address(blindArb));
    console.log("\nblindArb WETH initial balance = %s wei (WETH) [should be 1 ETH]\n", blindArbBalanceInitial);

    // Approve UniswapV3 router to spend PLU
    plu.approve(address(uniV3Router), PLU_SWAP_AMOUNT);

    uint256 uniV3PoolBalanceBeforeSwap = plu.balanceOf(address(uniV3Pool));
    console.log("uniV3Pool PLU balance before swap = %s wei (PLU)", uniV3PoolBalanceBeforeSwap);

    console.log("\n! simulate user swap to create arbitrage opportunity !\n");

    console.log("swap PLU -> WETH:");
    console.log("  tokenIn: %s (PLU)", address(plu));
    console.log("  tokenOut: %s (WETH)", address(weth));
    console.log("  amountIn: %s wei (PLU)", PLU_SWAP_AMOUNT);
    console.log("  fee: %s", FEE);

    // Prepare swap PLU -> WETH paramters
    ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
      tokenIn: address(plu),
      tokenOut: address(weth),
      fee: FEE, // 0.3%
      recipient: address(this),
      deadline: block.timestamp + 1000,
      amountIn: PLU_SWAP_AMOUNT,
      amountOutMinimum: 0, // Doesn't matter
      sqrtPriceLimitX96: 0 // No price limit
    });

    // Swap: PLU -> WETH
    uint256 wethAmountOut = uniV3Router.exactInputSingle(params);
    console.log("wethAmountOut = wei (WETH)", wethAmountOut);

    uint256 uniV3PoolBalanceAfterSwap = plu.balanceOf(address(uniV3Pool));
    uint256 uniV3PoolBalanceDeltaAfterSwap = uniV3PoolBalanceAfterSwap - uniV3PoolBalanceBeforeSwap;
    console.log("uniV3Pool PLU balance after swap = %s wei (PLU)", uniV3PoolBalanceAfterSwap);
    console.log("  +%s wei (PLU) [should be equal to amountIn]", uniV3PoolBalanceDeltaAfterSwap);

    // Now WETH costs more in UniswapV3 pool and PLU is cheaper
    // In UniswapV2 prices remain the same
    // Hence we got an arbitrage opportunity here: WETH -> PLU -> WETH

    console.log("\n! run blind arbitrage WETH -> PLU -> WETH !\n");

    blindArb.execute_weth_token0(
      address(uniV2Pair), // v2Pool
      address(uniV3Pool), // v3Pool
      WETH_ARB_AMOUNT,    // amountIn
      0                   // percentageToPayToCoinbase
    );

    uint256 blindArbBalanceFinal = weth.balanceOf(address(blindArb));
    int256 profit = int256(blindArbBalanceFinal) - int256(blindArbBalanceInitial);
    console.log("\nblindArb WETH balance after arbitrage = %s wei (WETH)", blindArbBalanceFinal);
    console.log("  profit: %s wei (WETH)", profit);

    // ~= 0.128 WETH ~= $464

    assertGt(profit, 0, "profit = 0, you suck");
  }
}

