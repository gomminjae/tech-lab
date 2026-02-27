// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title USDCEscrow — USDC 결제 에스크로 컨트랙트 (MVP)
/// @notice buyer가 USDC를 예치하고, 조건에 따라 seller에게 정산하거나 buyer에게 환불한다.
contract USDCEscrow is ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ─── 상태 정의 ─────────────────────────────────────────────
    // Deal의 생애주기를 나타내는 enum.
    // None은 사용하지 않지만, mapping 기본값(0)과 구분하기 위해 존재.
    enum Status {
        None,       // 0 — 미생성 (mapping 기본값)
        Created,    // 1 — 생성됨, 아직 예치 안 됨
        Deposited,  // 2 — buyer가 자금을 예치함
        Released,   // 3 — seller에게 정산 완료 (종결)
        Refunded    // 4 — buyer에게 환불 완료 (종결)
    }

    // ─── Deal 구조체 ───────────────────────────────────────────
    struct Deal {
        address buyer;
        address seller;
        IERC20 token;           // USDC 컨트랙트 주소
        uint256 amount;         // 에스크로 금액
        uint256 timeoutDuration; // timeout 기간(초) — createDeal에서 설정
        uint256 depositedAt;    // 예치 시점 (block.timestamp)
        uint256 timeoutAt;      // 이 시간 이후 buyer가 timeoutRefund 가능
        Status status;
    }

    // ─── 상태 변수 ─────────────────────────────────────────────
    uint256 public nextDealId;           // 자동 증가형 ID
    address public arbitrator;           // 중재자 (release/refund 권한)
    mapping(uint256 => Deal) public deals;

    // ─── 이벤트 ────────────────────────────────────────────────
    // indexed 키워드: 이벤트 로그를 필터링할 때 사용. EVM은 최대 3개까지 indexed 가능.
    event DealCreated(
        uint256 indexed dealId,
        address indexed buyer,
        address indexed seller,
        address token,
        uint256 amount,
        uint256 timeoutDuration
    );
    event Deposited(uint256 indexed dealId, address indexed buyer, uint256 amount);
    event Released(uint256 indexed dealId, address indexed seller, uint256 amount);
    event Refunded(uint256 indexed dealId, address indexed buyer, uint256 amount);

    // ─── 커스텀 에러 ───────────────────────────────────────────
    // revert 문자열 대신 custom error를 쓰면 가스비가 절약된다.
    error DealNotFound();
    error InvalidAmount();
    error InvalidState();
    error Unauthorized();
    error TimeoutNotReached();
    error ZeroAddress();
    error TimeoutTooShort();

    // ─── 생성자 ────────────────────────────────────────────────
    constructor(address _arbitrator) {
        if (_arbitrator == address(0)) revert ZeroAddress();
        arbitrator = _arbitrator;
    }

    // ─── 핵심 함수들 ───────────────────────────────────────────

    /// @notice 새 에스크로 거래를 생성한다.
    /// @param _seller 판매자 주소
    /// @param _token USDC 토큰 컨트랙트 주소
    /// @param _amount 에스크로 금액 (USDC 단위, 6 decimals)
    /// @param _timeoutDuration 예치 후 이 시간(초)이 지나면 buyer가 환불 가능
    /// @return dealId 생성된 거래 ID
    function createDeal(
        address _seller,
        address _token,
        uint256 _amount,
        uint256 _timeoutDuration
    ) external returns (uint256 dealId) {
        if (_seller == address(0)) revert ZeroAddress();
        if (_token == address(0)) revert ZeroAddress();
        if (_amount == 0) revert InvalidAmount();
        if (_timeoutDuration < 1 hours) revert TimeoutTooShort();

        dealId = nextDealId++;

        deals[dealId] = Deal({
            buyer: msg.sender,
            seller: _seller,
            token: IERC20(_token),
            amount: _amount,
            timeoutDuration: _timeoutDuration,
            depositedAt: 0,
            timeoutAt: 0,         // deposit 시점에 계산
            status: Status.Created
        });

        emit DealCreated(dealId, msg.sender, _seller, _token, _amount, _timeoutDuration);
    }

    /// @notice buyer가 USDC를 에스크로에 예치한다.
    /// @dev 호출 전에 buyer가 token.approve(escrow, amount)를 해야 한다.
    function deposit(uint256 _dealId) external nonReentrant {
        Deal storage deal = deals[_dealId];

        // Checks
        if (deal.status == Status.None) revert DealNotFound();
        if (deal.status != Status.Created) revert InvalidState();
        if (msg.sender != deal.buyer) revert Unauthorized();

        // Effects — 상태 변경을 외부 호출보다 먼저!
        deal.status = Status.Deposited;
        deal.depositedAt = block.timestamp;
        deal.timeoutAt = block.timestamp + deal.timeoutDuration;

        // Interactions — 외부 토큰 컨트랙트 호출
        deal.token.safeTransferFrom(msg.sender, address(this), deal.amount);

        emit Deposited(_dealId, msg.sender, deal.amount);
    }

    /// @notice 정산 — seller에게 자금을 전송한다.
    /// @dev buyer 또는 arbitrator만 호출 가능
    function release(uint256 _dealId) external nonReentrant {
        Deal storage deal = deals[_dealId];

        if (deal.status == Status.None) revert DealNotFound();
        if (deal.status != Status.Deposited) revert InvalidState();
        if (msg.sender != deal.buyer && msg.sender != arbitrator) revert Unauthorized();

        deal.status = Status.Released;

        deal.token.safeTransfer(deal.seller, deal.amount);

        emit Released(_dealId, deal.seller, deal.amount);
    }

    /// @notice 환불 — buyer에게 자금을 돌려준다.
    /// @dev seller 또는 arbitrator만 호출 가능
    function refund(uint256 _dealId) external nonReentrant {
        Deal storage deal = deals[_dealId];

        if (deal.status == Status.None) revert DealNotFound();
        if (deal.status != Status.Deposited) revert InvalidState();
        if (msg.sender != deal.seller && msg.sender != arbitrator) revert Unauthorized();

        deal.status = Status.Refunded;

        deal.token.safeTransfer(deal.buyer, deal.amount);

        emit Refunded(_dealId, deal.buyer, deal.amount);
    }

    /// @notice 시간 초과 환불 — timeout 이후 buyer가 직접 환불받는다.
    function timeoutRefund(uint256 _dealId) external nonReentrant {
        Deal storage deal = deals[_dealId];

        if (deal.status == Status.None) revert DealNotFound();
        if (deal.status != Status.Deposited) revert InvalidState();
        if (msg.sender != deal.buyer) revert Unauthorized();
        if (block.timestamp < deal.timeoutAt) revert TimeoutNotReached();

        deal.status = Status.Refunded;

        deal.token.safeTransfer(deal.buyer, deal.amount);

        emit Refunded(_dealId, deal.buyer, deal.amount);
    }

    // ─── 조회 함수 ─────────────────────────────────────────────

    /// @notice Deal 정보를 조회한다.
    function getDeal(uint256 _dealId)
        external
        view
        returns (
            address buyer,
            address seller,
            address token,
            uint256 amount,
            uint256 timeoutDuration,
            uint256 depositedAt,
            uint256 timeoutAt,
            Status status
        )
    {
        Deal storage deal = deals[_dealId];
        return (
            deal.buyer,
            deal.seller,
            address(deal.token),
            deal.amount,
            deal.timeoutDuration,
            deal.depositedAt,
            deal.timeoutAt,
            deal.status
        );
    }
}
