// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @notice 테스트용 Mock USDC. 실제 USDC처럼 6 decimals를 사용한다.
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {}

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    /// @dev 테스트 편의를 위해 누구나 mint 가능
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
