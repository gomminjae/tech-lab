// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

// ========== abstract contract ==========
// 단독 배포 불가. 자식이 반드시 구현해야 할 함수를 가짐
abstract contract Vehicle {
    string public color;

    constructor(string memory _color) {
        color = _color;
    }

    // virtual = 자식이 재정의 가능
    function honk() public pure virtual returns (string memory);

    function wheels() public pure virtual returns (uint256);
}

// ========== 단일 상속 ==========
contract Car is Vehicle {
    // TODO 1: 부모 생성자에 "Red" 전달
    // hint: constructor() Vehicle("값") { }
    constructor() Vehicle("Red") {
        
    }

    // TODO 2: honk() 구현 - "Beep!" 반환
    function honk() public pure override returns (string memory) {
        // 여기에 작성
        return "Beep!";
    }

    // TODO 3: wheels() 구현 - 4 반환
    function wheels() public pure override returns (uint256) {
        // 여기에 작성
        return 4;
    }
}

// ========== 다중 상속 ==========
// Solidity 특유: contract C is A, B 가능
abstract contract Electric {
    function batteryLevel() public pure virtual returns (uint256);
}

// TODO 4: Vehicle과 Electric을 동시에 상속
// hint: contract 이름 is 부모1, 부모2
contract Tesla is Vehicle, Electric {
    constructor() Vehicle("White") {}

    function honk() public pure override returns (string memory) {
        return "Honk!";
    }

    function wheels() public pure override returns (uint256) {
        return 4;
    }

    // TODO 5: batteryLevel() 구현 - 100 반환
    function batteryLevel() public pure override returns (uint256) {
        // 여기에 작성
        return 100;
    }
}
