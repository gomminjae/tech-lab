// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract PayableExample {
    address public owner;

    // 누가 얼마 입금했는지 기록
    mapping(address => uint256) public deposits;

    constructor() {
        owner = msg.sender;
    }

    // TODO 1: ETH 입금 함수 - payable 키워드가 있어야 ETH를 받을 수 있음
    // msg.value = 보낸 ETH 양
    function deposit() external payable {
        // 여기에 작성 - deposits[msg.sender]에 msg.value 누적
        deposits[msg.sender] += msg.value;
    }

    // TODO 2: 컨트랙트의 전체 ETH 잔액 조회
    function getBalance() external view returns (uint256) {
        // 여기에 작성 - address(this).balance 반환
        return address(this).balance;
    }

    // TODO 3: owner만 출금 가능 - call 방식 사용
    // hint: (bool success, ) = payable(owner).call{value: amount}("");
    function withdraw(uint256 amount) external {
        require(msg.sender == owner, "Not owner");
        require(amount <= address(this).balance, "Insufficient balance");
        // 여기에 작성 - call로 ETH 전송 + 실패 시 revert
        (bool success, ) = payable(owner).call{value: amount}("");
        require(success, "Transfer failed");
    }

    // TODO 4: receive - 함수 호출 없이 순수 ETH만 보냈을 때 실행
    // hint: receive() external payable { }
    // deposits[msg.sender]에 msg.value 누적
    receive() external payable {
        deposits[msg.sender] += msg.value;
    }


    // TODO 5: fallback - 존재하지 않는 함수 호출 시 실행
    // hint: fallback() external payable { }
    fallback() external payable {
    }
}
