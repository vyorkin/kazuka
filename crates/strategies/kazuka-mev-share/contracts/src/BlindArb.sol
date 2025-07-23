// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {IWETH} from "./interfaces/tokens/IWETH.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {ISwapRouter02} from "./interfaces/Uniswap.sol";

contract BlindArb {
  address public owner;
  IWETH internal immutable weth;
  IERC20 internal immutable inputToken;
  ISwapRouter02 internal immutable swapRouter;

  constructor (
    address swapRouterAddress,
    address wethAddress,
    address inputTokenAddress
  ) {
    owner = msg.sender;

    swapRouter = ISwapRouter02(swapRouterAddress);
    weth = IWETH(wethAddress);
    inputToken = IERC20(inputTokenAddress);
  }

  modifier onlyOwner() {
    require(msg.sender == owner, "Only owner");
    _;
  }

  /// @notice Simple owner-only test function
  function ownerOnlyFunction() external onlyOwner() {}

  /// @notice Deposit received ETH into WETH
  function depositWETH() external payable {
    require(msg.value > 0, "BlindArb: No ETH sent");
    weth.deposit{value: msg.value}();
  }

  function flashSwap(address uniV3PoolAddress, uint256 amountOut) external onlyOwner {
  }

  /// @notice Uniswap V3 callback for swaps
  function uniswapV3SwapCallback(
    int256 amount0Delta,
    int256 amount1Delta,
    bytes calldata data
  ) external {
  }

  receive() external payable {}
}
