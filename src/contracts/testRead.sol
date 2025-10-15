// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract TestRead {
  uint256 public value = 100;

  function read() public view returns (uint256) {
    return value;
  }
}