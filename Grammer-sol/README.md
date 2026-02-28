# Solidity Grammar Study

Foundry 기반 Solidity 문법 학습 프로젝트

## 학습 목차

| Ch | 파일 | 주제 | 핵심 키워드 |
|----|------|------|------------|
| 01 | `src/01_DataTypes.sol` | 자료형 | bool, uint, int, address, string, array, mapping, struct, enum |
| 02 | `src/02_Functions.sol` | 함수와 가시성 | public, external, internal, private, view, pure |
| 03 | `src/03_Interface.sol` | 인터페이스 | interface, is, override, 타입으로 활용 |
| 04 | `src/04_Inheritance.sol` | 상속 | abstract, is, virtual, override, 다중 상속 |
| 05 | `src/05_Modifier.sol` | 제어자와 에러 | modifier, require, revert, custom error |
| 06 | `src/06_Storage.sol` | 데이터 위치 | storage, memory, calldata |
| 07 | `src/07_Payable.sol` | ETH 송수신 | payable, msg.value, call, receive, fallback |
| 08 | `src/08_Events.sol` | 이벤트 | event, emit, indexed |

## 핵심 요약

### 가시성 (Visibility)
- `public` — 내부 + 외부 모두 호출 가능
- `external` — 외부에서만 호출 가능
- `internal` — 현재 + 자식 컨트랙트만
- `private` — 현재 컨트랙트만

### 상태 변경 여부
- `view` — 상태 읽기만 (가스비 무료)
- `pure` — 상태 접근 X, 순수 계산 (가스비 무료)
- (없음) — 상태 변경 가능 (가스비 발생)

### 데이터 위치
- `storage` — 블록체인 영구 저장 (비쌈), 참조 시 원본 변경
- `memory` — 함수 내 임시 (저렴), 복사본이라 원본 안 바뀜
- `calldata` — 읽기 전용 입력 (가장 저렴)

### 컨트랙트 종류
- `interface` — 설계도 (함수 형태만, 구현 X)
- `abstract` — 미완성 (일부 구현, 배포 X)
- `contract` — 완성품 (배포 가능)

### ETH 수신
- `receive()` — 순수 ETH 전송 시 자동 실행
- `fallback()` — 없는 함수 호출 시 실행

## 빌드

```shell
forge build
```
