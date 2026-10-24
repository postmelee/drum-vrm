# Run and check locally

**English** · [한국어](local-development.ko.md)

Use this guide to reproduce the application locally. The commands are instructions, not a record of tests already passing. See [feature availability](../reference/availability.md) for differences from the public site.

## Prerequisites

- Node.js 24 or newer; CI uses the pinned version in the [workflow](../../.github/workflows/ci.yml).
- Rust from [rust-toolchain.toml](../../rust-toolchain.toml), with the `wasm32-unknown-unknown` target.
- wasm-pack 0.15.0.

From the repository root:

```sh
npm ci
cargo install wasm-pack --version 0.15.0 --locked
npm run build:wasm
npm run dev
```

Open `/drum-vrm/` on the local URL printed by Vite. Rebuild WASM after Rust changes. `src/wasm/` is generated; npm and Rust versions are locked by their lockfiles.

## Check a change

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

License generation requires installed npm packages and the locked Rust dependency graph; fetch or build the Rust dependencies first. Production files go to `dist/` and use the `/drum-vrm/` base path. Previewing the static build is distinct from deploying it.

For document changes, follow the [documentation policy](../../specs/policies/documentation.md) and [validation policy](../../specs/policies/validation.md). Current task evidence belongs in [work records](../../mydocs/README.md). Private MIDI, WAV, VRM and local research downloads stay outside tracked files; `.local/` is ignored.

[Task guides](README.md)
