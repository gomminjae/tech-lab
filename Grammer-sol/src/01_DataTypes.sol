// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract DataTypes {
    // ========== 값 타입 (Value Types) ==========

    bool public isActive = true;        // true / false
    uint256 public count = 100;         // 부호 없는 정수 (0 ~ 2^256-1)
    int256 public temperature = -10;    // 부호 있는 정수
    address public owner = msg.sender;  // 20바이트 지갑/컨트랙트 주소

    // ========== 참조 타입 (Reference Types) ==========

    string public greeting = "Hello";           // 문자열
    uint256[] public scores;                    // 동적 배열 (크기 가변)
    uint256[3] public medals = [1, 2, 3];       // 고정 배열 (크기 3)

    // ========== Mapping ==========
    // key => value 저장소 (JS의 Map, Python의 dict와 비슷)
    mapping(address => uint256) public balances;

    // ========== Struct ==========
    // 여러 값을 하나로 묶는 커스텀 타입
    struct User {
        string name;
        uint256 score;
    }
    mapping(address => User) public users;

    // ========== Enum ==========
    // 정해진 선택지 중 하나 (내부적으로 0, 1, 2...)
    enum Status { Pending, Active, Closed }
    Status public status = Status.Pending;
}
