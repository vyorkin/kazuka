// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";
// import "forge-std/StdUtils.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {console} from "forge-std/console.sol";

import {BlindArb} from "../src/BlindArb.sol";
import {IWETH} from "../src/interfaces/tokens/IWETH.sol";
import {ISwapRouter02, IUniswapV3Pool} from "../src/interfaces/Uniswap.sol";

contract BlindArbTest is Test {
  IWETH internal constant weth = IWETH(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
  IERC20 internal constant plu = IERC20(0xD8912C10681D8B21Fd3742244f44658dBA12264E);

  ISwapRouter02 internal constant swapRouter = ISwapRouter02(0xE592427A0AEce92De3Edee1F18E0157C05861564);
  IUniswapV3Pool internal constant uniV3Pool = IUniswapV3Pool(0xe11Ee9c18d03B43d6A7fC53e51AeDda8451e837A);

  uint256 internal constant PLU_SWAP_AMOUNT = 1000 ether;

  BlindArb internal blindArb;

  function setUp() public {
    vm.createSelectFork(
      vm.envString("ALCHEMY_ETHEREUM_MAINNET_RPC_URL"),
      vm.envUint("FORK_BLOCK_NUMBER")
    );

    blindArb = new BlindArb(address(swapRouter), address(weth), address(plu));

    // Get some PLU for this test contract
    deal(address(plu), address(this), PLU_SWAP_AMOUNT);
    // Get some PLU for the BlindArb contract
    // so it can repay flash swap in callback
    deal(address(plu), address(blindArb), 10 ether);
  }

  function test_OwnerIsSet() public view {
    assertEq(blindArb.owner(), address(this), "Owner should be deployer");
  }

  function test_OwnerOnlyFunction() public {
    blindArb.ownerOnlyFunction();
  }

  function test_RevertWhen_NonOwnerCall() public {
    vm.prank(address(0xBEEF));
    vm.expectRevert(bytes("Only owner"));
    blindArb.ownerOnlyFunction();
  }

  function test_DepositWETH() public {
    // Send ETH to BlindArb contract.
    // Simulate user depositing ETH.

    uint256 ethAmount = 1 ether;

    (bool sent, ) = address(blindArb).call{value: ethAmount}("");
    require(sent, "Failed to send ETH");

    // Call depositWETH to convert: ETH -> WETH
    blindArb.depositWETH{value: ethAmount}();

    uint256 wethBalance = weth.balanceOf(address(blindArb));
    assertEq(wethBalance, ethAmount, "WETH balance should equal deposited ETH");
  }

  function test_Swap_PLU_WETH() public {
    // Approve Uniswap V3 router to spend test contract's PLU
    plu.approve(address(swapRouter), PLU_SWAP_AMOUNT);

    // Swap: PLU -> WETH
    ISwapRouter02.ExactInputSingleParams memory params =
      ISwapRouter02.ExactInputSingleParams({
        tokenIn: address(plu),
        tokenOut: address(weth),
        fee: 3000, // 0.3% fee
        recipient: address(this),
        deadline: block.timestamp + 1000,
        amountIn: PLU_SWAP_AMOUNT,
        amountOutMinimum: 0, // any output amount
        sqrtPriceLimitX96: 0 // no limit
      });
      uint256 wethAmountOut = swapRouter.exactInputSingle(params);

      console.log("wethAmountOut = %s = %s WETH", wethAmountOut, wethAmountOut / 1 ether);

      // Ensure we got some WETH in return for PLU
      assertGt(wethAmountOut, 0, "Swap output must be greater than zero");
  }

  function test_flashSwapFiresCallback() public {
    blindArb.flashSwap(address(uniV3Pool), 1 ether);
  }
}

