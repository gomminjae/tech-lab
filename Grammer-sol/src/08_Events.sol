// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract SimpleToken {
    mapping(address => uint256) public balances;

    // TODO 1: Transfer 이벤트 선언
    // hint: event 이름(타입 indexed 파라미터, ...);
    // from과 to는 indexed, amount는 indexed 아님
    event Transfer(address indexed from, address indexed to, uint256 amount);

    // TODO 2: Mint 이벤트 선언
    // hint: 누구에게(indexed) 얼마를 발행했는지
    event Mint(address indexed to, uint256 amount);

    // TODO 3: 토큰 발행 - balances 증가 + Mint 이벤트 emit
    function mint(address to, uint256 amount) external {
        // 여기에 작성
        balances[to] += amount;
        emit Mint(to, amount);
    }

    // TODO 4: 토큰 전송 - 잔액 체크 + 이동 + Transfer 이벤트 emit
    function transfer(address to, uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        // 여기에 작성 - 보내는 사람 차감, 받는 사람 증가, emit
        balances[msg.sender] -= amount;
        balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
    }
}
