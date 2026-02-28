// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract StorageExample {
    struct User {
        string name;
        uint256 score;
    }

    mapping(address => User) public users;
    uint256[] public scores;

    // ========== calldata vs memory ==========

    // TODO 1: calldata 사용 - 읽기 전용, 가스 가장 저렴
    // external 함수의 매개변수에 적합
    function register(string calldata _name) external {
        // 여기에 작성 - users[msg.sender]에 User 저장 (score는 0)
        users[msg.sender] = User(_name,0);
    }

    // TODO 2: memory 사용 - 함수 내에서 수정이 필요할 때
    function formatName(string memory _name) public pure returns (string memory) {
        // memory라서 함수 내에서 수정 가능 (calldata는 불가)
        // 여기서는 그냥 반환
        return _name;
    }

    // ========== storage 참조 vs memory 복사 ==========

    // TODO 3: storage 참조 - 원본을 직접 수정
    function updateScore(uint256 _score) external {
        // 여기에 작성 - User storage로 참조 가져와서 score 변경
        // hint: User storage user = users[msg.sender];
        User storage user = users[msg.sender];
        user.score = _score;
    }

    // TODO 4: memory 복사 - 원본에 영향 없음 (흔한 실수!)
    function buggyUpdate(uint256 _score) external {
        // 이렇게 하면 원본이 안 바뀜! (memory는 복사본)
        User memory user = users[msg.sender];
        user.score = _score;
        // 함수 끝나면 사라짐... users[msg.sender].score는 그대로
    }

    // ========== 배열과 storage ==========

    // TODO 5: 배열에 push
    function addScore(uint256 _score) external {
        // 여기에 작성 - scores 배열에 _score 추가
        scores.push(_score);
    }

    // TODO 6: 배열 길이 반환 (view)
    function getScoreCount() external view returns (uint256) {
        // 여기에 작성
        return scores.length;
    }
}
