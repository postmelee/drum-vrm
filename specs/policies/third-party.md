# Third-party adoption policy

## Separate the material and intended use

Treat papers, source code, datasets, trained weights, motion recordings, 3D models,
textures, sounds and original license text as separate materials. A paper or public
URL does not grant rights to every associated artifact. Video-use permission does
not establish permission to redistribute the source model.

The project's own code is MIT. Preserve each third-party license and any required
notice/source offer. Do not relabel external material as MIT or imply that a source
author endorses the project. Keep model import metadata and source-specific additional
terms visible where needed for the user's intended use.

## Source and asset records

Use stable research IDs with author/institution, original URL, material kind, reviewed
version/commit/date, relevant locator, rights evidence and intended adoption scope.
Separate what a source teaches from the project's interpretation and from measured
implementation results. The [reader source guide](../../docs/reference/sources/README.md)
provides navigation; historical research remains under `mydocs/tech/research/`.

Bundled assets require a source URL, creator, exact acquired identity/checksum, license
text, required credit and transformation record beside the asset. The provenance JSON
and original license are authoritative. Derived preset guidance and distribution
notices must not maintain conflicting hand-written copies of those facts.

No upload, original conversation, privately downloaded avatar or research dataset
enters public source/tests/build merely because it was used for investigation.
Store local research inputs in ignored `.local/` and use authored public fixtures.

## Dependencies and notices

Pin adopted package/tool versions and source identities through lockfiles and the
recorded toolchain. [scripts/licenses.mjs](../../scripts/licenses.mjs) generates
`THIRD_PARTY_NOTICES.md`, `public/licenses.txt` and the locked dependency reference.
Change generated paths with the generator; never repair a generated file by hand.

Preserve original license text without translating, shortening or splitting it to
meet prose line limits. The current generator can reuse matching Rust license text
from `public/licenses.txt` when toolchain documentation is unavailable; that is a
verified input fallback, not a reason to treat arbitrary output as trusted input.
Keep this behavior explicit until a separately reviewed input-store change replaces it.

MPL-covered dependencies require their applicable notices and corresponding source
links. Check actual included files and modifications rather than extending a package
summary to all optional codecs or assets. Research conditions and unresolved rights
remain explicit until the required evidence is obtained.

## Adoption check

Before bundling new material, record the exact files, rights for the intended action,
credit/notice duties, transformations and reproducible source identity. Update the
relevant source/provenance record and generated distribution notice, then validate
that the build contains only the authorized material. Acquisition for local evaluation
and inclusion in a public artifact are separate actions.
