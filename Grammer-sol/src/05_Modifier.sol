// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract Vault {
    address public owner;
    uint256 public balance;
    bool public locked;

    // 커스텀 에러 - revert와 함께 사용, 가스 효율적
    error NotOwner(address caller);
    error VaultLocked();
    error InsufficientBalance(uint256 requested, uint256 available);

    constructor() {
        owner = msg.sender;
    }

    // TODO 1: modifier 작성 - owner만 통과
    // hint: require(조건, "에러메시지") + _; 잊지 마세요
    modifier onlyOwner() {
        // 여기에 작성
        require(msg.sender == owner, "Not owner");
        _;
    }

    // TODO 2: modifier 작성 - locked가 true이면 차단
    // hint: 커스텀 에러 사용해보기 → if (locked) revert VaultLocked();
    modifier notLocked() {
        // 여기에 작성
        if(locked) revert VaultLocked();
        _;
    }

    // TODO 3: modifier 2개 동시 적용 - onlyOwner + notLocked
    function deposit(uint256 amount) public onlyOwner notLocked {
        balance += amount;
        
    }

    // TODO 4: require로 잔액 체크 후 출금
    function withdraw(uint256 amount) public onlyOwner notLocked {
        // 여기에 작성 - balance보다 많이 출금하면 revert InsufficientBalance
        if(amount > balance) revert InsufficientBalance(amount, balance);
        balance -= amount;
    }

    // TODO 5: lock/unlock 토글
    function toggleLock() public onlyOwner {
        // 여기에 작성 - locked를 반전
        locked = !locked;
    }
}
