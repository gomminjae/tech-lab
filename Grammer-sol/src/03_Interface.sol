// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

// ========== Interface 정의 ==========
// "동물이라면 반드시 이 함수들이 있어야 한다"
interface IAnimal {
    function speak() external view returns (string memory);
    function legs() external view returns (uint256);
}

// ========== Interface 구현 ==========
contract Dog is IAnimal {
    // TODO 1: speak() 구현 - "Woof" 반환
    // hint: interface 함수를 구현할 때 override 키워드 필요
    function speak() external pure override returns (string memory) {
        // 여기에 작성
        return "Woof";
    }

    // TODO 2: legs() 구현 - 4 반환
    function legs() external pure override returns (uint256) {
        // 여기에 작성
        return 4;
    }
}

contract Cat is IAnimal {
    // TODO 3: speak() 구현 - "Meow" 반환
    function speak() external pure override returns (string memory) {
        // 여기에 작성
        return "Meow";
    }

    // TODO 4: legs() 구현 - 4 반환
    function legs() external pure override returns (uint256) {
        // 여기에 작성
        return 4;
    }
}

// ========== Interface를 타입으로 활용 ==========
contract AnimalShelter {
    // TODO 5: IAnimal 타입으로 받으면 Dog든 Cat든 상관없이 호출 가능
    // 이게 interface의 진짜 힘!
    function getSound(IAnimal animal) external view returns (string memory) {
        // 여기에 작성 -
        return animal.speak();
    }
}
