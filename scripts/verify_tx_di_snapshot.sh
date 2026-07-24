#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
vernal_root="$(cd "$script_dir/.." && pwd)"
default_workspace_root="$(cd "$vernal_root/.." && pwd)"
tx_di_root="${1:-$default_workspace_root/tx_di}"
expected_commit="2a5dae2e2573bdd679f7bbe882fbfea18ae51fdc"

if [[ ! -d "$tx_di_root/.git" ]]; then
    echo "tx-di repository not found: $tx_di_root" >&2
    exit 1
fi

actual_commit="$(git -C "$tx_di_root" rev-parse HEAD)"
if [[ "$actual_commit" != "$expected_commit" ]]; then
    echo "tx-di commit mismatch: expected $expected_commit, got $actual_commit" >&2
    exit 1
fi

compare_file() {
    local source_file="$1"
    local target_file="$2"

    if ! cmp -s "$source_file" "$target_file"; then
        echo "snapshot mismatch: $source_file -> $target_file" >&2
        exit 1
    fi
}

compare_file \
    "$tx_di_root/tx-di-core/src/lib.rs" \
    "$vernal_root/third-party/tx-di/tx-di-core/src/lib.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/error.rs" \
    "$vernal_root/crates/vernal-core/src/error.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/component.rs" \
    "$vernal_root/crates/vernal-ioc/src/component.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/registry.rs" \
    "$vernal_root/crates/vernal-ioc/src/registry.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/scope.rs" \
    "$vernal_root/crates/vernal-ioc/src/scope.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/store.rs" \
    "$vernal_root/crates/vernal-ioc/src/store.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/topology.rs" \
    "$vernal_root/crates/vernal-ioc/src/topology.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/aop.rs" \
    "$vernal_root/crates/vernal-aop/src/aop.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/config.rs" \
    "$vernal_root/crates/vernal-context/src/config.rs"
compare_file \
    "$tx_di_root/tx-di-core/src/lifecycle.rs" \
    "$vernal_root/crates/vernal-context/src/lifecycle.rs"
compare_file \
    "$tx_di_root/tx-di-core/Cargo.toml" \
    "$vernal_root/third-party/tx-di/manifests/tx-di-core.Cargo.toml"
compare_file \
    "$tx_di_root/tx-di-macros/Cargo.toml" \
    "$vernal_root/third-party/tx-di/manifests/tx-di-macros.Cargo.toml"
compare_file \
    "$tx_di_root/LICENSE" \
    "$vernal_root/third-party/tx-di/LICENSE"

macro_file_count=0
while IFS= read -r source_file; do
    relative_path="${source_file#"$tx_di_root/tx-di-macros/"}"
    target_file="$vernal_root/crates/vernal-macros/upstream/tx-di/tx-di-macros/$relative_path"
    compare_file "$source_file" "$target_file"
    macro_file_count=$((macro_file_count + 1))
done < <(find "$tx_di_root/tx-di-macros/src" -type f -name '*.rs' | sort)

if [[ "$macro_file_count" -ne 16 ]]; then
    echo "unexpected tx-di macro source count: $macro_file_count" >&2
    exit 1
fi

echo "tx-di snapshot verified: commit=$actual_commit core_files=10 macro_files=$macro_file_count"
