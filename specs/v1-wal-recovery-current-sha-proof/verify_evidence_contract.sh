#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
feature_dir="$repo_root/specs/v1-wal-recovery-current-sha-proof"
report="$feature_dir/final_report.md"

fail() {
  echo "$*" >&2
  exit 1
}

if [[ ! -f "$report" ]]; then
  fail "missing required current-run evidence report: $report"
fi

live_head="$(git -C "$repo_root" rev-parse HEAD)"
live_status="$(git -C "$repo_root" status --short)"
live_parent="$(git -C "$repo_root" rev-parse HEAD^ 2>/dev/null || true)"

block() {
  local heading="$1"
  awk -v heading="$heading" '
    $0 == heading { in_block = 1; print; next }
    in_block && /^### EV-/ { exit }
    in_block { print }
  ' "$report"
}

require_block() {
  local heading="$1"
  local content
  content="$(block "$heading")"
  [[ -n "$content" ]] || fail "missing evidence block: $heading"
  printf '%s\n' "$content"
}

require_fixed() {
  local heading="$1"
  local needle="$2"
  local content
  content="$(require_block "$heading")"
  grep -Fq "$needle" <<<"$content" || fail "$heading missing required text: $needle"
}

require_regex() {
  local heading="$1"
  local regex="$2"
  local message="$3"
  local content
  content="$(require_block "$heading")"
  grep -Eq "$regex" <<<"$content" || fail "$heading $message"
}

field_value() {
  local heading="$1"
  local field="$2"
  local content
  content="$(require_block "$heading")"
  sed -n "s/^$field:[[:space:]]*//p" <<<"$content" | head -n 1
}

first_transcript_path() {
  local heading="$1"
  local transcript_field
  transcript_field="$(field_value "$heading" "transcripts")"
  [[ -n "$transcript_field" ]] || fail "$heading missing transcripts field"
  local first_path="${transcript_field%%,*}"
  first_path="${first_path#"${first_path%%[![:space:]]*}"}"
  first_path="${first_path%"${first_path##*[![:space:]]}"}"
  [[ "$first_path" != /* ]] && first_path="$feature_dir/$first_path"
  printf '%s\n' "$first_path"
}

fenced_stdout() {
  local heading="$1"
  local content
  content="$(require_block "$heading")"
  awk '
    $0 == "stdout:" { after_stdout = 1; next }
    after_stdout && $0 == "```" && !in_fence { in_fence = 1; next }
    in_fence && $0 == "```" { exit }
    in_fence { print }
  ' <<<"$content"
}

require_fixed "# WAL Recovery Current-SHA Evidence" "# WAL Recovery Current-SHA Evidence"

require_fixed "### EV-PROVENANCE" "implementation_active_run_id:"
require_fixed "### EV-PROVENANCE" "implementation_result_path:"
run_id="$(field_value "### EV-PROVENANCE" "implementation_active_run_id")"
result_path="$(field_value "### EV-PROVENANCE" "implementation_result_path")"
[[ -n "$run_id" ]] || fail "### EV-PROVENANCE missing implementation_active_run_id value"
[[ -n "$result_path" ]] || fail "### EV-PROVENANCE missing implementation_result_path value"
[[ "$result_path" == *"$run_id"* ]] || fail "### EV-PROVENANCE result path does not contain implementation run id"
[[ -f "$result_path" ]] || fail "### EV-PROVENANCE implementation result path does not exist: $result_path"

require_fixed "### EV-IDENTITY-HEAD" "command: git rev-parse HEAD"
require_regex "### EV-IDENTITY-HEAD" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_regex "### EV-IDENTITY-HEAD" 'stdout:[[:space:]]*"?[0-9a-f]{40}"?' "must record a 40-hex SHA stdout"
recorded_head="$(field_value "### EV-IDENTITY-HEAD" "stdout" | tr -d '"')"
if [[ "$recorded_head" != "$live_head" && "$recorded_head" != "$live_parent" ]]; then
  fail "### EV-IDENTITY-HEAD must match live HEAD or the immediate pre-final commit HEAD"
fi
head_stdout_path="$(first_transcript_path "### EV-IDENTITY-HEAD")"
[[ -f "$head_stdout_path" ]] || fail "### EV-IDENTITY-HEAD transcript missing: $head_stdout_path"
head_transcript="$(tr -d '\n' < "$head_stdout_path")"
[[ "$head_transcript" == "$recorded_head" ]] || fail "### EV-IDENTITY-HEAD transcript does not match inline stdout"
require_fixed "### EV-IDENTITY-HEAD" "stderr:"

require_fixed "### EV-IDENTITY-STATUS" "command: git status --short"
require_regex "### EV-IDENTITY-STATUS" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_fixed "### EV-IDENTITY-STATUS" "stdout:"
require_fixed "### EV-IDENTITY-STATUS" "stderr:"
recorded_status="$(fenced_stdout "### EV-IDENTITY-STATUS")"
status_stdout_path="$(first_transcript_path "### EV-IDENTITY-STATUS")"
[[ -f "$status_stdout_path" ]] || fail "### EV-IDENTITY-STATUS transcript missing: $status_stdout_path"
status_transcript="$(cat "$status_stdout_path")"
[[ "$status_transcript" == "$recorded_status" ]] || fail "### EV-IDENTITY-STATUS transcript does not match inline stdout"

require_fixed "### EV-TEST-WAL" "command: cargo test --test wal_recovery"
require_regex "### EV-TEST-WAL" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_fixed "### EV-TEST-WAL" "committed_wal_replay_survives_reopen_via_cli"
require_fixed "### EV-TEST-WAL" "rolled_back_wal_frame_is_not_replayed_as_uncommitted_change"
require_fixed "### EV-TEST-WAL" "incomplete_wal_entry_is_not_replayed_without_public_rollback_cli"
require_fixed "### EV-TEST-WAL" "committed_frame_after_incomplete_tail_cleanup_remains_replayable"
require_fixed "### EV-TEST-WAL" "committed_wal_frame_ahead_of_page_store_fails_deterministically"

require_fixed "### EV-VERIFY-BASE" "command: ./scripts/verify"
require_regex "### EV-VERIFY-BASE" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_fixed "### EV-VERIFY-BASE" "cargo fmt --check"
require_fixed "### EV-VERIFY-BASE" "cargo clippy --all-targets -- -D warnings"
require_fixed "### EV-VERIFY-BASE" "cargo test"
require_fixed "### EV-VERIFY-BASE" "cargo run --bin db -- --help"

require_fixed "### EV-SMOKE-CREATE-INSERT" 'command: cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, '\''ada'\''); INSERT INTO users VALUES (2, '\''bea'\'');"'
require_regex "### EV-SMOKE-CREATE-INSERT" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_fixed "### EV-SMOKE-CREATE-INSERT" 'stdout: ""'
require_fixed "### EV-SMOKE-CREATE-INSERT" 'stderr: ""'

require_fixed "### EV-WAL-AFTER-CREATE-INSERT" "exists: true"
require_regex "### EV-WAL-AFTER-CREATE-INSERT" 'byte_length:[[:space:]]*[1-9][0-9]*' "must record positive byte_length"

require_fixed "### EV-SMOKE-REOPEN-SELECT" 'command: cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"'
require_regex "### EV-SMOKE-REOPEN-SELECT" 'exit_code:[[:space:]]*0' "must record exit_code: 0"
require_fixed "### EV-SMOKE-REOPEN-SELECT" 'stderr: ""'
require_fixed "### EV-SMOKE-REOPEN-SELECT" 'stdout: "id|name\n1|ada\n2|bea\n"'

require_fixed "### EV-WAL-AFTER-REOPEN-SELECT" "exists: true"
require_regex "### EV-WAL-AFTER-REOPEN-SELECT" 'byte_length:[[:space:]]*[1-9][0-9]*' "must record positive byte_length"

require_fixed "### EV-FIXTURE-RATIONALE" "V1-observable"
require_fixed "### EV-FIXTURE-RATIONALE" "no public rollback"
require_fixed "### EV-FIXTURE-RATIONALE" "incomplete"
require_fixed "### EV-FIXTURE-RATIONALE" "uncommitted"

require_fixed "### EV-ACCEPTANCE-MAPPING" "gap-v1-transaction-wal-recovery"
require_fixed "### EV-ACCEPTANCE-MAPPING" "gate-v1-transactions-wal-recovery"
require_fixed "### EV-ACCEPTANCE-MAPPING" "req-v1-wal-recovery-proof"

echo "evidence contract shape ok"
