#!/usr/bin/env bash
# Delegate a small, test-verifiable coding task to the local Ollama model.
#
# Runs aider inside the dev container (where cargo lives). aider's own
# edit -> cargo test -> fix loop stays local; this script prints only a compact
# summary so the supervising agent does not pay for the intermediate rounds.
# Full transcripts go to .local-coder/runs/<timestamp>/.
#
# Usage:
#   tools/local-coder/delegate.sh [options] -p "task" FILE...
#   tools/local-coder/delegate.sh [options] -F task.md FILE...
#
# FILE paths are relative to the repo root; aider may edit (or create) them.
#
# Options:
#   -p TEXT   task description
#   -F FILE   read task description from FILE
#   -r FILE   read-only context file for aider (repeatable)
#   -t CMD    test command (default: from .aider.conf.yml, i.e. whole workspace)
#   -m MODEL  Ollama model (default: $LOCAL_CODER_MODEL or qwen3-coder:30b)
#   -T SECS   timeout for the aider run (default: $LOCAL_CODER_TIMEOUT or 900)
#
# Exit status: 0 = tests green, 1 = tests red / aider failed, 2 = usage/setup error.
set -euo pipefail

usage() { sed -n '2,/^set -euo/p' "$0" | sed '$d; s/^# \{0,1\}//'; exit 2; }

model="${LOCAL_CODER_MODEL:-qwen3-coder:30b}"
timeout_secs="${LOCAL_CODER_TIMEOUT:-900}"
default_test_cmd="cargo test --manifest-path impls/rust/Cargo.toml"
task="" test_cmd="" read_only=()

while getopts "p:F:r:t:m:T:h" opt; do
  case "$opt" in
    p) task="$OPTARG" ;;
    F) task="$(cat "$OPTARG")" ;;
    r) read_only+=("$OPTARG") ;;
    t) test_cmd="$OPTARG" ;;
    m) model="$OPTARG" ;;
    T) timeout_secs="$OPTARG" ;;
    *) usage ;;
  esac
done
shift $((OPTIND - 1))
[[ -n "$task" && $# -gt 0 ]] || usage
files=("$@")

repo="$(git rev-parse --show-toplevel)"
cd "$repo"

# Works both on the host (via docker exec into the dev container) and inside it.
exec_prefix=()
if [[ ! -f /.dockerenv ]]; then
  container="$(docker ps -q --filter "label=devcontainer.local_folder=$repo" | head -n1)"
  if [[ -z "$container" ]]; then
    echo "error: dev container for $repo is not running (open the folder in VS Code / 'Reopen in Container')" >&2
    exit 2
  fi
  workdir="$(docker inspect -f "{{range .Mounts}}{{if eq .Source \"$repo\"}}{{.Destination}}{{end}}{{end}}" "$container")"
  exec_prefix=(docker exec -u vscode -w "$workdir" "$container")
fi
dexec() { "${exec_prefix[@]}" "$@"; }
if ! dexec sh -c 'command -v aider' >/dev/null; then
  echo "error: aider not found in the dev container (rebuild the container)" >&2
  exit 2
fi
if ! curl -sf http://127.0.0.1:11434/api/version >/dev/null; then
  echo "error: Ollama is not reachable on 127.0.0.1:11434" >&2
  exit 2
fi

run_id="$(date +%Y%m%d-%H%M%S)"
run_dir=".local-coder/runs/$run_id"
mkdir -p "$run_dir"
printf '%s\n' "$task" > "$run_dir/task.md"

# Backup of the target files that already exist, for a one-command rollback.
existing=() created=()
for f in "${files[@]}"; do
  if [[ -e "$f" ]]; then existing+=("$f"); else created+=("$f"); fi
done
if ((${#existing[@]})); then
  tar -cf "$run_dir/backup.tar" "${existing[@]}"
fi

aider_args=(--model "ollama_chat/$model" --message "$task")
for r in "${read_only[@]}"; do aider_args+=(--read "$r"); done
[[ -n "$test_cmd" ]] && aider_args+=(--test-cmd "$test_cmd")

start=$(date +%s)
aider_status=0
timeout "$timeout_secs" "${exec_prefix[@]}" \
  aider "${aider_args[@]}" "${files[@]}" > "$run_dir/aider.log" 2>&1 || aider_status=$?
duration=$(( $(date +%s) - start ))

# Verify independently of what aider claims.
test_status=0
dexec bash -lc "${test_cmd:-$default_test_cmd}" > "$run_dir/test.log" 2>&1 || test_status=$?

result="green"
(( test_status != 0 )) && result="red"
(( aider_status == 124 )) && result="timeout"
(( aider_status != 0 && aider_status != 124 )) && result="aider-error"
[[ "$result" == "aider-error" && $test_status -eq 0 ]] && result="green"

mkdir -p .local-coder
[[ -f .local-coder/runs.tsv ]] || printf 'run\tmodel\tseconds\tresult\tfiles\n' > .local-coder/runs.tsv
printf '%s\t%s\t%s\t%s\t%s\n' "$run_id" "$model" "$duration" "$result" "${files[*]}" >> .local-coder/runs.tsv

echo "local-coder: result=$result model=$model time=${duration}s run=$run_dir"
echo "--- changes"
git diff --stat -- "${files[@]}" || true
git ls-files --others --exclude-standard -- "${files[@]}" | sed 's/^/new: /'
if [[ "$result" != "green" ]]; then
  echo "--- test failures (excerpt; full log: $run_dir/test.log)"
  grep -E '^(error(\[E[0-9]+\])?:|warning: unused)|^ *--> |panicked at|^test .* FAILED|^failures:' "$run_dir/test.log" \
    | head -n 40 || tail -n 20 "$run_dir/test.log"
  rollback=()
  [[ -f "$run_dir/backup.tar" ]] && rollback+=("tar -xf $run_dir/backup.tar")
  ((${#created[@]})) && rollback+=("rm -f ${created[*]}")
  ((${#rollback[@]})) && echo "--- rollback: $(IFS=';'; echo "${rollback[*]}" | sed 's/;/; /g')"
fi

[[ "$result" == "green" ]]
