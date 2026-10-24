# 실제 도입 의존성 잠금과 라이선스

이 파일과 배포 고지는 `npm run licenses`로 생성한다. 네트워크 조회에 의존하지 않고 설치된 파일·lockfile·`cargo metadata --locked --offline`을 대조한다. 버전이나 체크섬이 맞지 않거나 원본 라이선스 파일이 없으면 실패한다. 절대 로컬 경로는 배포 문서에 포함하지 않는다.

## 브라우저 패키지

production 의존성의 전이 폐쇄를 기록한다. `@types/` 두 패키지는 타입 선언이며 브라우저 실행 코드가 아니다. 개발·검사 도구(Vite, TypeScript, Vitest, @types/three 등)는 `package-lock.json`의 별도 dev 의존성으로 고정하며 아래 런타임 목록과 구분한다.

| 패키지 | 버전 | 라이선스 | 정확한 소스 | lock integrity |
|---|---|---|---|---|
| @types/dom-mediacapture-transform | 0.1.12 | MIT | [archive](https://registry.npmjs.org/@types/dom-mediacapture-transform/-/dom-mediacapture-transform-0.1.12.tgz) | `sha512-d7/QsLRwF864A5mgIM/YrfiglHoYn7zgCcAoJgW404r+2DwnNr7EBbLnCWpmOMgH8y0te73L1AV6H1bmauaWFw==` |
| @types/dom-webcodecs | 0.1.13 | MIT | [archive](https://registry.npmjs.org/@types/dom-webcodecs/-/dom-webcodecs-0.1.13.tgz) | `sha512-O5hkiFIcjjszPIYyUSyvScyvrBoV3NOEEZx/pMlsu44TKzWNkLVBBxnxJz42in5n3QIolYOcBYFCPZZ0h8SkwQ==` |
| mediabunny | 1.62.1 | MPL-2.0 | [archive](https://registry.npmjs.org/mediabunny/-/mediabunny-1.62.1.tgz) | `sha512-lYzeerDwo9QLhBCipqQDXOSV7InvaIFplllZQviPO5rPwH+IUr8M1uvumk1olIUgMRGkPiInRriv25WFErUgMw==` |
| three | 0.186.1 | MIT | [archive](https://registry.npmjs.org/three/-/three-0.186.1.tgz) | `sha512-blFeqb49wRCSGUGj7gtpfnSGHy2lwDk94RhUmS1c/hTby70kvChbWpkJ4Pm1390LqzzvTmzgXKHPEafJwCb8jA==` |

## Rust 패키지

`wasm32-unknown-unknown` 기준 Cargo 해석 그래프의 registry 패키지다. proc-macro/build 구성요소도 보수적으로 포함하며 모두가 WASM에 링크된다는 뜻은 아니다. crate 원본 archive와 Cargo.lock SHA-256을 제공한다. Rust 표준 라이브러리는 고정된 `rust-toolchain.toml`의 툴체인 구성요소로 사용한다.

| crate | 버전 | 라이선스 | 정확한 소스 | Cargo.lock checksum |
|---|---|---|---|---|
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/bumpalo/3.20.3/download) | `sha256:72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649` |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/cfg-if/1.0.5/download) | `sha256:4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600` |
| itoa | 1.0.18 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/itoa/1.0.18/download) | `sha256:8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682` |
| memchr | 2.8.3 | Unlicense OR MIT | [archive](https://crates.io/api/v1/crates/memchr/2.8.3/download) | `sha256:cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98` |
| midly | 0.5.3 | Unlicense | [archive](https://crates.io/api/v1/crates/midly/0.5.3/download) | `sha256:207d755f4cb882d20c4da58d707ca9130a0c9bc5061f657a4f299b8e36362b7a` |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/once_cell/1.21.4/download) | `sha256:9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50` |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/proc-macro2/1.0.107/download) | `sha256:985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9` |
| quote | 1.0.47 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/quote/1.0.47/download) | `sha256:1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001` |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/rustversion/1.0.23/download) | `sha256:cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f` |
| serde | 1.0.229 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/serde/1.0.229/download) | `sha256:4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba` |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/serde_core/1.0.229/download) | `sha256:67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48` |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/serde_derive/1.0.229/download) | `sha256:e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348` |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/serde_json/1.0.154/download) | `sha256:e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6` |
| syn | 2.0.119 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/syn/2.0.119/download) | `sha256:872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297` |
| syn | 3.0.9 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/syn/3.0.9/download) | `sha256:d78c8dee4c7bf0e14673097256fed6142ce9d3b85a408189d07482442145823b` |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [archive](https://crates.io/api/v1/crates/unicode-ident/1.0.27/download) | `sha256:a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563` |
| wasm-bindgen | 0.2.127 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/wasm-bindgen/0.2.127/download) | `sha256:1b70935747edd64d89de3efa29d73789b806c15798f8e7dca4d8ac356b50ce70` |
| wasm-bindgen-macro | 0.2.127 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/wasm-bindgen-macro/0.2.127/download) | `sha256:77775f8f3f7217702089053b94958f8f54061a3f663417df76e19cbdcca29bc1` |
| wasm-bindgen-macro-support | 0.2.127 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/wasm-bindgen-macro-support/0.2.127/download) | `sha256:e11d33f857dc2fb11b8bc75aee111aa9cbeb12cd9f25efd3d4c2a3dd4e235284` |
| wasm-bindgen-shared | 0.2.127 | MIT OR Apache-2.0 | [archive](https://crates.io/api/v1/crates/wasm-bindgen-shared/0.2.127/download) | `sha256:7ef64dbcc55df09c7e5a46182d181c2cfa3e925f3da937ea764728b4bbb9dcbf` |
| zmij | 1.0.23 | MIT | [archive](https://crates.io/api/v1/crates/zmij/1.0.23/download) | `sha256:29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b` |

## 배포 조건과 범위

- 원본 라이선스/저작권 문구는 [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md)와 배포물 `licenses.txt`에 그대로 포함한다. dual-license crate는 원본에 있는 양쪽 라이선스 모두 보존한다.
- Mediabunny의 MPL-2.0 covered source는 수정하지 않았다. 정확한 npm archive에는 `src/`와 원본 LICENSE가 포함되며, 사용자가 별도 비용 없이 해당 버전의 대응 소스를 받을 수 있다. 자체 앱 파일은 MIT다.
- 외부 샘플·캐릭터·텍스처·모션·데이터·가중치는 도입하지 않았다. 참고 조사 링크는 실행 코드 도입과 별개다.
- 이 고지는 실제 설치 패키지와 잠금 체크섬을 근거로 한다. 확인하지 않은 Git tag/commit은 기재하지 않는다. 버전 변경 시 설치·잠금·고지를 함께 갱신한다.

## 원본 고지 예외와 툴체인

midly 0.5.3 crate의 include 목록은 LICENSE를 누락한다. crate 내부 `.cargo_vcs_info.json`의 commit `816cb2a585b4d5d1fd7aeb0cb35668f21dc00869`를 확인한 뒤 [그 commit의 LICENSE](https://raw.githubusercontent.com/negamartin/midly/816cb2a585b4d5d1fd7aeb0cb35668f21dc00869/LICENSE) 원문을 생성기에 보관했다. 버전·commit·license 표현식이 모두 같을 때만 이 예외를 적용한다.

Rust 1.94.1의 표준 라이브러리 고지도 설치 툴체인의 `COPYRIGHT-library.html`에서 추출하여 함께 배포한다. 소스 commit은 `rustc --version --verbose`에서 확인한 `e408947bfd200af42db322daf0fadfe7e26d3bd1`다. rust-docs가 없는 최소 CI 환경에서는 이 릴리스와 commit이 정확히 같은 기존 배포 고지만 재사용한다. 새로운 툴체인은 rust-docs를 설치해 다시 생성해야 한다.
