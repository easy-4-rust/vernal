# tx-di source snapshot

This directory records the provenance of the unmodified tx-di sources imported
into Vernal.

- Upstream repository: `https://gitee.com/tian_xiong/tx_di`
- Local source repository: sibling workspace `tx_di`
- Upstream commit: `2a5dae2e2573bdd679f7bbe882fbfea18ae51fdc`
- Commit date: `2026-07-23`
- License: MIT, copyright 2026 田雄
- Import mode: byte-for-byte source snapshot
- Compile status: copied implementation files are not declared by Vernal crate roots

The imported implementation files are preserved without symbol renaming,
import rewriting, formatting, or dependency substitution. They are not
declared from the Vernal crate roots yet. The original monolithic crate root
and macro tree remain in the snapshot area because compiling them directly
would recreate the tx-di crate layout.

## Module mapping

| tx-di source | Vernal ownership boundary | Snapshot destination |
|:---|:---|:---|
| `tx-di-core/src/component.rs` | IoC component contract | `crates/vernal-beans/src/component.rs` |
| `tx-di-core/src/registry.rs` | IoC link-time registry | `crates/vernal-beans/src/registry.rs` |
| `tx-di-core/src/scope.rs` | IoC scope model | `crates/vernal-beans/src/scope.rs` |
| `tx-di-core/src/store.rs` | IoC component store | `crates/vernal-beans/src/store.rs` |
| `tx-di-core/src/topology.rs` | IoC dependency topology | `crates/vernal-beans/src/topology.rs` |
| `tx-di-core/src/aop.rs` | AOP runtime | `crates/vernal-aop/src/aop.rs` |
| `tx-di-core/src/config.rs` | Context configuration loading | `crates/vernal-context/src/config.rs` |
| `tx-di-core/src/lifecycle.rs` | Application context and lifecycle | `crates/vernal-context/src/lifecycle.rs` |
| `tx-di-core/src/error.rs` | Shared error bridge | `crates/vernal-core/src/error.rs` |
| `tx-di-core/src/lib.rs` | Original monolithic crate root | `third-party/tx-di/tx-di-core/src/lib.rs` |
| `tx-di-macros/src/**` | Component and interception macros | `crates/vernal-macros/upstream/tx-di/tx-di-macros/src/**` |

The original core and macro manifests are retained under `manifests/`; the
original MIT license is retained as `LICENSE`.

## Why the snapshot is not compiled

The upstream files currently depend on the original monolithic crate layout,
including `tx_error`, `tx_common`, `linkme`, `DashMap`, Tokio, TOML loading,
global state, and `tx_di_core` paths emitted by macros. Compiling the files in
their target Vernal crates would require changing the source and would violate
the requested byte-for-byte import.

The next migration stage must therefore:

1. preserve this snapshot;
2. declare or adapt one copied capability at a time from the Vernal crate roots;
3. replace tx-di-specific error and utility dependencies with Vernal contracts;
4. add tests before declaring the copied capability implemented;
5. record behavioral differences from the upstream source.

Run `scripts/verify_tx_di_snapshot.sh` from any directory to verify the sibling
source repository, commit, file set, and byte equality.
