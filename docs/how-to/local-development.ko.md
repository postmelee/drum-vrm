# 로컬 실행과 검사

[English](local-development.md) · **한국어**

앱을 로컬에서 재현하는 안내입니다. 아래 명령은 실행 절차이며 이미 검사를 통과했다는 기록이 아닙니다. 공개 사이트와의 차이는 [기능 제공 범위](../reference/availability.ko.md)를 참고하세요.

## 준비물

- Node.js 24 이상. CI의 고정 버전은 [workflow](../../.github/workflows/ci.yml)에 있습니다.
- [rust-toolchain.toml](../../rust-toolchain.toml)의 Rust와 `wasm32-unknown-unknown` target.
- wasm-pack 0.15.0.

저장소 루트에서 실행합니다.

```sh
npm ci
cargo install wasm-pack --version 0.15.0 --locked
npm run build:wasm
npm run dev
```

Vite가 출력한 로컬 주소의 `/drum-vrm/`을 엽니다. Rust를 변경하면 WASM을 다시 빌드하세요. `src/wasm/`은 생성물이며 npm·Rust 버전은 각 lockfile로 고정합니다.

## 변경 검사

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run typecheck
npm test
npm run licenses
npm run build
npm run preview
```

라이선스 생성에는 설치된 npm 패키지와 잠금 Rust 의존성 그래프가 필요하므로 Rust 의존성을 먼저 가져오거나 빌드합니다. production 파일은 `dist/`에 생성되며 `/drum-vrm/` base를 사용합니다. 정적 빌드 미리보기와 배포는 다른 작업입니다.

문서를 변경할 때는 [문서 정책](../../specs/policies/documentation.md)과 [검증 정책](../../specs/policies/validation.md)을 따르세요. 현재 작업의 근거는 [작업 기록](../../mydocs/README.md)에 둡니다. 개인 MIDI·WAV·VRM과 로컬 조사 다운로드는 추적 파일에 넣지 않으며 `.local/`은 무시됩니다.

[작업별 사용법](README.ko.md)
