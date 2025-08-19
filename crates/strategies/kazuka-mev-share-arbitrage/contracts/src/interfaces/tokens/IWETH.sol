// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

interface IWETH {
  function deposit() external payable;
  function balanceOf(address) external view returns (uint256);
  function transfer(address, uint256) external returns (bool);
  function withdraw(uint256) external;
}
