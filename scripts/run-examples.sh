#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf '%s\n' 'Usage: bash scripts/run-examples.sh [--pattern GLOB] [--skip-build]'
}

pattern='*.prim'
skip_build=false
while (($# > 0)); do
    case "$1" in
        --pattern)
            if (($# < 2)) || [[ -z "$2" ]]; then
                printf '%s\n' '[ERROR] --pattern requires a nonempty filename pattern.' >&2
                exit 1
            fi
            pattern=$2
            shift 2
            ;;
        --skip-build)
            skip_build=true
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            printf '[ERROR] Unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

repository_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repository_root"

# WSLとWindowsで同じ作業ツリーを使っても、生成物は別の場所へ置きます。
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$repository_root/target/unix"}
primer="$CARGO_TARGET_DIR/debug/primer"
examples=()
for example in "$repository_root"/examples/*; do
    # 右辺は引用せず、ファイル名に対するBashのパターンとして比較します。
    if [[ -f "$example" && "${example##*/}" == $pattern ]]; then
        examples+=("$example")
    fi
done

if ((${#examples[@]} == 0)); then
    printf "[ERROR] No examples match pattern '%s'.\n" "$pattern" >&2
    exit 1
fi

if [[ "$skip_build" == false ]]; then
    printf '%s\n' 'Building Primer...'
    if ! cargo build --quiet; then
        printf '%s\n' '[ERROR] cargo build failed.' >&2
        exit 1
    fi
fi

if [[ ! -f "$primer" || ! -x "$primer" ]]; then
    printf '%s\n' '[ERROR] Primer executable not found. Run without --skip-build.' >&2
    exit 1
fi

passed=0
failed=()
for example in "${examples[@]}"; do
    name=${example##*/}
    printf '\n=== %s ===\n' "$name"
    if "$primer" run "$example"; then
        passed=$((passed + 1))
        printf '[PASS] %s\n' "$name"
    else
        failed+=("$name")
        printf '[FAIL] %s\n' "$name" >&2
    fi
done

printf '\n=== Summary ===\nPassed: %d\nFailed: %d\n' "$passed" "${#failed[@]}"
if ((${#failed[@]} > 0)); then
    printf 'Failed example: %s\n' "${failed[@]}" >&2
    exit 1
fi
