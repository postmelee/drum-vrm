# Dependencies and licenses

English · [한국어](README.ko.md)

Drum VRM's own code is [MIT licensed](../../../LICENSE). Libraries, models, and their source materials retain their own terms.

Start with the [locked dependency inventory](locked.md) for exact package versions, source archives, and checksums. Its field labels are English and its data is shared by both language guides.

| Record | What it contains |
|---|---|
| [Repository notices](../../../THIRD_PARTY_NOTICES.md) | Full application notices and attribution for repository tooling |
| [Distribution notices](../../../public/licenses.txt) | Full runtime and asset notices shipped with the web application |
| [Bundled models](../../../public/models/README.md) | Model creators, source links, original terms, and provenance records |

## Inventory scope

- npm: the production dependency closure from `package-lock.json` and the installed packages. Declaration packages under `@types/` are identified separately from runtime code. Build and test tools remain pinned in the lockfile but are not listed as application runtime dependencies.
- Rust: the locked registry graph for `wasm32-unknown-unknown`, including build and procedural-macro dependencies conservatively. Listing a crate does not imply every part is linked into the final WASM. Standard-library notices come from the pinned toolchain.
- Models: each `public/models/*-provenance.json` owns its source, creator, terms, original file hashes, and runtime adaptation description. The generated `NOTICE.txt` and distribution notices use those records.
- Hyper-Waterfall: operating templates are repository tooling. Their pinned source and license are recorded in `.hyper-waterfall/version.json` and the repository notices; they are not part of the browser distribution.

## Corresponding source and original notices

Mediabunny's MPL-2.0 covered source files are used without modification. The inventory and notices link the exact npm source archive, including `src/` and its original license. The application's separate MIT files do not change those terms.

The published `midly` crate omits its license file. The generator preserves the original text from the recorded upstream commit and accepts that exception only when the crate version, source commit, and license expression match. The [inventory](locked.md#source-evidence-exceptions) contains that source pin.

The Rust standard-library notice is extracted from the installed toolchain's `COPYRIGHT-library.html`. A minimal CI toolchain without `rust-docs` can reuse the existing distribution notice only when **both the compiler release and commit match**. A new toolchain requires its own original notice; install `rust-docs` and regenerate it.

## Regenerating the records

With the locked dependencies and pinned Rust toolchain installed, run:

```sh
npm run licenses
```

The command uses installed license files, `package-lock.json`, `Cargo.lock`, and `cargo metadata --locked --offline`. Missing source evidence, dependency mismatches, or changed model/license bytes fail the check. It writes the inventory, repository and distribution notices, and model notice from those inputs.

Original license texts remain in their supplied language. They are preserved in full rather than rewritten or translated. Research citations alone do not mean a library, model, motion dataset, audio sample, or model weight is included in the application.
