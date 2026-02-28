// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract Functions {
    uint256 public count = 0;

    // TODO 1: pure 함수 - 두 수를 더해서 반환
    // hint: 블록체인 데이터를 전혀 안 쓰니까 pure
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        // 여기에 작성
        return a + b;
    }

    // TODO 2: view 함수 - count 값을 반환
    // hint: 상태변수를 "읽기만" 하니까 view
    function getCount() public view returns (uint256) {
        // 여기에 작성
        return count;
    }

    // TODO 3: 상태 변경 함수 - count를 1 증가
    // hint: 상태를 바꾸니까 view/pure 안 붙임
    function increment() public {
        // 여기에 작성
        count += 1;
    }

    // TODO 4: external 함수 - count를 원하는 값으로 설정
    // hint: external은 외부에서만 호출 가능 (컨트랙트 내부에서 this.setCount()으로만 가능)
    function setCount(uint256 _newCount) external {
        // 여기에 작성
        count = _newCount;
    }

    // TODO 5: internal 함수 - count를 0으로 리셋 (외부 호출 불가)
    // hint: internal은 이 컨트랙트 + 자식 컨트랙트만 호출 가능
    function _reset() internal {
        // 여기에 작성
        count = 0;
    }

    // TODO 6: 위의 _reset()을 호출하는 public 함수
    function resetCount() public {
        // 여기에서 _reset() 호출
        _reset();
    }
}

