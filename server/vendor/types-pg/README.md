# pg declaration snapshot

The npm registry is unavailable in the development sandbox. This local dev
dependency contains the genuine DefinitelyTyped declarations for the installed
`pg` 8.23 driver, fetched through the read-only GitHub connection. It has no
runtime JavaScript, installation scripts or custom replacement declarations.

Source: [DefinitelyTyped, immutable commit
97ba786e647c0899a2dd6d1b6807896762e725fa](https://github.com/DefinitelyTyped/DefinitelyTyped/tree/97ba786e647c0899a2dd6d1b6807896762e725fa/types/pg).
The source package version `8.23.9999` identifies the upstream development
snapshot; it is not a claimed npm release.

`index.d.ts`, `index.d.mts`, both `lib` declarations and
`UPSTREAM.package.json` are unchanged copies. Their Git blob SHA-1 values were
verified against the upstream directory listing. `SOURCE.json` records those
values and SHA-256 checksums, the immutable commit and upstream license hash.
The upstream MIT license and package owner attribution are retained.

Local `package.json` supplies installable packaging metadata, preserves the
upstream conditional exports and declaration dependencies, and excludes its
repository-only workspace dev dependency. The root npm manifest and generated
lockfile use `file:vendor/types-pg`. Runtime `pg` and its locked dependencies are
unchanged; the declarations reuse the installed Node, pg-protocol and pg-types
packages. Strict NodeNext typing remains enabled without ambient `any` shims or
`skipLibCheck`.

Run `npm run types:verify` to check the stored bytes and packaging, then
`npm run typecheck`. A clean install must include `vendor/types-pg` in the source
checkout. When registry access returns, this snapshot can be replaced through a
normal reviewed npm installation of the matching published `@types/pg` package;
regenerate the lockfile and rerun the complete backend checks.
