# session-cleaner

[English](README.md) | [한국어](README.ko-kr.md)

> Codex, Claude Code, OMP 세션 파일 중 가치가 낮은 파일을 찾고, 안전한 후보만 플랫폼 Trash로 이동하는 보수적인 로컬 CLI입니다.

## 특징

- Codex, Claude Code, OMP 세션 JSONL 저장소 스캔
- 단순 line count가 아니라 provider별 의미 기반 classifier 사용
- 기본 동작은 dry-run scan
- 단일 세션 파일이 보존/정리 후보로 분류된 이유 설명
- 고신뢰 후보를 플랫폼 Trash로 이동; 영구 삭제 명령은 아직 없음
- 변경 로그를 `${XDG_STATE_HOME:-~/.local/state}/session-cleaner/runs/` 아래에 기록
- 사람용 text 출력과 자동화용 JSON 출력 지원
- custom root와 age threshold를 위한 선택적 TOML config 지원

## 설치

### 소스에서 빌드

Rust 1.94+가 필요합니다.

```bash
cargo build --release
cp target/release/session-cleaner ~/.local/bin/session-cleaner
```

설치된 바이너리 확인:

```bash
session-cleaner --version
session-cleaner --help
```

## 빠른 시작

### 1. 파일을 변경하지 않고 스캔

```bash
session-cleaner scan --older-than 24h
```

`scan`은 기본적으로 안전합니다. 후보를 출력할 뿐 파일을 이동하거나 삭제하지 않습니다.

### 2. 정리 전에 단일 파일 확인

```bash
session-cleaner explain /path/to/session.jsonl
```

경로만으로 provider가 명확하지 않으면 `--provider`를 지정하세요.

```bash
session-cleaner explain /path/to/session.jsonl --provider claude
```

### 3. 고신뢰 후보를 Trash로 이동

```bash
session-cleaner trash --older-than 24h --yes
```

> [!CAUTION]
> `trash`는 파일을 플랫폼 Trash로 이동합니다. 일반적인 데스크톱 환경에서는 복구 가능하지만, 파일시스템을 실제로 변경합니다. 먼저 `scan`을 실행하세요.

## 명령 개요

| 명령 | 목적 |
|------|------|
| `session-cleaner scan` | 정리 후보를 출력합니다. 기본 동작은 dry-run입니다. |
| `session-cleaner scan --json` | 자동화용 JSON 스캔 리포트를 출력합니다. |
| `session-cleaner explain <path>` | 단일 세션 파일의 분류 이유를 설명합니다. |
| `session-cleaner trash --yes` | 고신뢰 후보를 플랫폼 Trash로 이동합니다. |
| `session-cleaner --version` | 패키지 버전을 출력합니다. |

## 기본 Root

| Provider | 기본 root |
|----------|-----------|
| Codex | `~/.codex/sessions`, `~/.codex/archived_sessions` |
| Claude Code | `~/.claude/projects` |
| OMP | `~/.omp/agent/sessions` |

Claude Code 스캔은 recursive라 nested subagent/workflow JSONL 파일도 검사합니다. OMP는 고신뢰 trivial pattern이 확인될 때까지 기본적으로 report-only입니다.

## Custom Root

`--root` 또는 `-r`에는 `provider=path` 형식을 사용합니다.

```bash
session-cleaner scan -r codex=/tmp/codex-sessions --older-than 7d
session-cleaner trash -r claude=/tmp/claude-projects --older-than 7d --yes
```

지원 provider:

- `codex`
- `claude`
- `omp`

## 설정

`--config` 또는 `-c`로 TOML config를 전달할 수 있습니다.

```toml
older_than = "24h"

[roots]
codex = ["~/.codex/sessions", "~/.codex/archived_sessions"]
claude = ["~/.claude/projects"]
omp = ["~/.omp/agent/sessions"]
```

실행:

```bash
session-cleaner scan --config ./session-cleaner.toml
session-cleaner trash --config ./session-cleaner.toml --yes
```

root와 age threshold는 CLI flag가 config 값보다 우선합니다.

## 안전 모델

- `scan`은 파일을 변경하지 않습니다.
- `trash`는 `--yes` 또는 `-y`가 필요합니다.
- 최근 파일은 `--older-than`으로 보호합니다. 기본값은 `24h`입니다.
- 모호하거나, 파싱되지 않거나, 지원되지 않거나, 신뢰도가 낮은 파일은 보존합니다.
- 정리 규칙은 provider별로 나뉘며 의도적으로 보수적입니다.
- 영구 삭제는 구현되어 있지 않습니다.

## 분류 예시

고신뢰 정리 후보 예시:

- metadata와 startup event만 있고 유의미한 user/assistant 작업이 없는 Codex bootstrap-only 세션
- 유일한 response item이 user `<turn_aborted>` marker인 Codex turn-aborted 세션
- assistant/tool 작업이 없는 Claude Code interrupted-only 세션
- `/exit`와 `Bye!` 같은 completion stdout만 있는 Claude Code exit-only 세션

보존되는 세션 예시:

- 사용자 작업 내용이 있는 파일
- assistant 내용이 있는 파일
- tool call 또는 tool result가 있는 파일
- Claude workflow journal 또는 의미가 불명확한 nested 파일
- OMP 파일. 향후 안전한 pattern이 추가되기 전까지 보존합니다.

## JSON 출력

```bash
session-cleaner scan --json --older-than 24h
```

agent나 script가 provider, decision, reason, confidence, byte size, inspected line count, event types 같은 안정적인 필드를 필요로 할 때 JSON 출력을 사용하세요.

## 개발

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## 프로젝트 상태

이 도구는 로컬 maintenance CLI입니다. 현재 구현은 공격적인 삭제보다 안전하고 되돌릴 수 있는 정리를 우선합니다.
