// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {USDCEscrow} from "../src/USDCEscrow.sol";
import {MockUSDC} from "./mocks/MockERC20.sol";

contract USDCEscrowTest is Test {
    USDCEscrow public escrow;
    MockUSDC public usdc;

    address public arbitrator = makeAddr("arbitrator");
    address public buyer = makeAddr("buyer");
    address public seller = makeAddr("seller");
    address public stranger = makeAddr("stranger");

    uint256 public constant DEAL_AMOUNT = 100e6; // 100 USDC (6 decimals)
    uint256 public constant TIMEOUT = 1 hours;

    function setUp() public {
        escrow = new USDCEscrow(arbitrator);
        usdc = new MockUSDC();

        // buyer에게 USDC 지급
        usdc.mint(buyer, 1000e6);
    }

    // ─── 환경 확인 테스트 ──────────────────────────────────────

    function test_setUp() public view {
        assertEq(escrow.arbitrator(), arbitrator);
        assertEq(usdc.decimals(), 6);
        assertEq(usdc.balanceOf(buyer), 1000e6);
    }

    // ─── createDeal 테스트 ─────────────────────────────────────

    function test_createDeal_success() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(seller, address(usdc), DEAL_AMOUNT, TIMEOUT);

        assertEq(dealId, 0);
        assertEq(escrow.nextDealId(), 1);

        (
            address _buyer,
            address _seller,
            address _token,
            uint256 _amount,
            uint256 _timeoutDuration,
            ,
            ,
            USDCEscrow.Status _status
        ) = escrow.getDeal(dealId);

        assertEq(_buyer, buyer);
        assertEq(_seller, seller);
        assertEq(_token, address(usdc));
        assertEq(_amount, DEAL_AMOUNT);
        assertEq(_timeoutDuration, TIMEOUT);
        assertEq(uint8(_status), uint8(USDCEscrow.Status.Created));
    }

    function test_createDeal_revert_zeroSeller() public {
        vm.prank(buyer);
        vm.expectRevert(USDCEscrow.ZeroAddress.selector);
        escrow.createDeal(address(0), address(usdc), DEAL_AMOUNT, TIMEOUT);
    }

    function test_createDeal_revert_zeroToken() public {
        vm.prank(buyer);
        vm.expectRevert(USDCEscrow.ZeroAddress.selector);
        escrow.createDeal(seller, address(0), DEAL_AMOUNT, TIMEOUT);
    }

    function test_createDeal_revert_zeroAmount() public {
        vm.prank(buyer);
        vm.expectRevert(USDCEscrow.InvalidAmount.selector);
        escrow.createDeal(seller, address(usdc), 0, TIMEOUT);
    }

    function test_createDeal_revert_timeoutTooShort() public {
        vm.prank(buyer);
        vm.expectRevert(USDCEscrow.TimeoutTooShort.selector);
        escrow.createDeal(seller, address(usdc), DEAL_AMOUNT, 30 minutes);
    }

    // ─── deposit 테스트 ────────────────────────────────────────

    function test_deposit_success() public {
        // 1. Deal 생성
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(seller, address(usdc), DEAL_AMOUNT, TIMEOUT);

        // 2. buyer가 approve
        vm.prank(buyer);
        usdc.approve(address(escrow), DEAL_AMOUNT);

        // 3. buyer가 deposit
        vm.prank(buyer);
        escrow.deposit(dealId);

        // 4. 검증
        (, , , , , , , USDCEscrow.Status status) = escrow.getDeal(dealId);
        assertEq(uint8(status), uint8(USDCEscrow.Status.Deposited));
        assertEq(usdc.balanceOf(address(escrow)), DEAL_AMOUNT);
        assertEq(usdc.balanceOf(buyer), 1000e6 - DEAL_AMOUNT);
    }
}
