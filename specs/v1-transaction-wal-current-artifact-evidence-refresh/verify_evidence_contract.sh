#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
feature_dir="$repo_root/specs/v1-transaction-wal-current-artifact-evidence-refresh"
evidence_dir="$feature_dir/evidence"

fail() {
  echo "$*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required evidence artifact: $path"
}

require_text() {
  local path="$1"
  local needle="$2"
  grep -Fq "$needle" "$path" || fail "$path missing required text: $needle"
}

require_regex() {
  local path="$1"
  local regex="$2"
  local message="$3"
  grep -Eq "$regex" "$path" || fail "$path $message"
}

reject_text() {
  local path="$1"
  local needle="$2"
  local message="$3"
  if grep -Fq "$needle" "$path"; then
    fail "$path $message"
  fi
}

command_block() {
  local path="$1"
  local command="$2"
  awk -v needle="command: $command" '
    $0 == needle {
      capture = 1
      found = 1
      print
      next
    }
    capture && /^command: / {
      exit
    }
    capture && /^## / {
      exit
    }
    capture {
      print
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "$path"
}

require_command_exit_zero() {
  local path="$1"
  local command="$2"
  local block
  block="$(command_block "$path" "$command" || true)"
  [[ -n "$block" ]] || fail "$path missing structured command evidence: $command"
  grep -Eq 'exit_code:[[:space:]]*0' <<<"$block" || fail "$path command does not have adjacent exit_code: 0 evidence: $command"
}

require_requirement_row_field() {
  local path="$1"
  local requirement="$2"
  local field="$3"
  local block
  block="$(grep -F -A 24 "$requirement" "$path" || true)"
  [[ -n "$block" ]] || fail "$path missing requirement row: $requirement"
  grep -Fq "$field" <<<"$block" || fail "$path $requirement missing required field: $field"
}

reject_scheduler_identity_values() {
  local path="$1"
  [[ ! -f "$path" ]] && return 0
  if grep -Eq 'active_run_id|qa_prep_(exec|retry)_[[:alnum:]_-]+|plan_(exec|retry)_[[:alnum:]_-]+|impl_(exec|retry)_[[:alnum:]_-]+|code_review_(exec|retry)_[[:alnum:]_-]+|final_(exec|retry)_[[:alnum:]_-]+' "$path"; then
    fail "$path contains scheduler/control-plane identity text that is not allowed in implementation-owned product evidence"
  fi
}

require_file "$evidence_dir/current-repo-sha.txt"
require_file "$evidence_dir/command-log.md"
require_file "$evidence_dir/requirement-evidence.md"
require_file "$evidence_dir/wal-sidecar-smoke.md"
require_file "$evidence_dir/crash-matrix-log.md"
require_file "$feature_dir/final_review.md"

require_text "$evidence_dir/current-repo-sha.txt" "command: git rev-parse HEAD"
require_text "$evidence_dir/current-repo-sha.txt" "command: git status --short"
evidence_sha="$(awk '
  $0 == "command: git rev-parse HEAD" {
    in_block = 1
    next
  }
  in_block && /^stdout: / {
    print $2
    exit
  }
' "$evidence_dir/current-repo-sha.txt")"
[[ "$evidence_sha" =~ ^[0-9a-f]{40}$ ]] || fail "$evidence_dir/current-repo-sha.txt must record a 40-character git SHA stdout for git rev-parse HEAD"
git -C "$repo_root" cat-file -e "$evidence_sha^{commit}" || fail "$evidence_dir/current-repo-sha.txt records a SHA that is not present as a local commit: $evidence_sha"
for command in \
  "git rev-parse HEAD" \
  "git status --short" \
  "test -f tests/wal_recovery.rs" \
  "test -f tests/crash_matrix.rs" \
  "test -x scripts/verify" \
  "test -x scripts/verify_crash_matrix"; do
  require_command_exit_zero "$evidence_dir/current-repo-sha.txt" "$command"
done
for required_path in \
  "path: tests/wal_recovery.rs" \
  "path: tests/crash_matrix.rs" \
  "path: scripts/verify" \
  "path: scripts/verify_crash_matrix"; do
  require_text "$evidence_dir/current-repo-sha.txt" "$required_path"
done
for sha_bound_artifact in \
  "$evidence_dir/command-log.md" \
  "$evidence_dir/requirement-evidence.md" \
  "$evidence_dir/wal-sidecar-smoke.md" \
  "$evidence_dir/crash-matrix-log.md" \
  "$feature_dir/final_review.md"; do
  require_text "$sha_bound_artifact" "$evidence_sha"
done

for command in \
  "scripts/verify" \
  "cargo test --test wal_recovery" \
  "cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli" \
  "cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change" \
  "cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli" \
  "cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable" \
  "cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically" \
  "scripts/verify_crash_matrix"; do
  require_text "$evidence_dir/command-log.md" "$command"
  require_command_exit_zero "$evidence_dir/command-log.md" "$command"
done

for req in \
  "REQ-8-begin-commit-and-rollback-provide-44e7901f" \
  "REQ-8-committed-writes-survive-crash-and-35caf667" \
  "REQ-9-provide-wal-or-equivalent-write-80297892" \
  "REQ-9-recovery-must-be-idempotent-and-300531dc" \
  "REQ-9-checkpoint-or-log-truncation-must-d633d286"; do
  require_text "$evidence_dir/requirement-evidence.md" "$req"
  require_text "$feature_dir/final_review.md" "$req"
  require_requirement_row_field "$evidence_dir/requirement-evidence.md" "$req" "command:"
  require_requirement_row_field "$evidence_dir/requirement-evidence.md" "$req" "expected_behavior:"
  require_requirement_row_field "$evidence_dir/requirement-evidence.md" "$req" "observed_result:"
  require_requirement_row_field "$evidence_dir/requirement-evidence.md" "$req" "artifact_refs:"
  require_requirement_row_field "$evidence_dir/requirement-evidence.md" "$req" "blocker_status:"
done

require_text "$evidence_dir/wal-sidecar-smoke.md" 'command: cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, '\''ada'\''); INSERT INTO users VALUES (2, '\''bea'\'');"'
require_text "$evidence_dir/wal-sidecar-smoke.md" 'command: cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"'
require_text "$evidence_dir/wal-sidecar-smoke.md" 'stdout: "id|name\n1|ada\n2|bea\n"'
require_text "$evidence_dir/wal-sidecar-smoke.md" "sidecar_path: \"\$DB_PATH.wal\""
require_text "$evidence_dir/wal-sidecar-smoke.md" "exists_after_create_insert: true"
require_text "$evidence_dir/wal-sidecar-smoke.md" "exists_after_reopen_select: true"
require_regex "$evidence_dir/wal-sidecar-smoke.md" 'bytes_after_create_insert:[[:space:]]*[1-9][0-9]*' "must record positive WAL bytes after create/insert"
require_regex "$evidence_dir/wal-sidecar-smoke.md" 'bytes_after_reopen_select:[[:space:]]*[1-9][0-9]*' "must record positive WAL bytes after reopen/select"
reject_text "$evidence_dir/wal-sidecar-smoke.md" "command: target/debug/db" "must record build-coupled cargo run product smoke, not direct target/debug/db execution"
reject_text "$evidence_dir/requirement-evidence.md" "command: target/debug/db" "must map WAL smoke evidence to build-coupled cargo run product smoke"

for case_id in CM-001 CM-002 CM-003 CM-004 CM-005 CM-006; do
  require_text "$evidence_dir/crash-matrix-log.md" "$case_id"
done
require_text "$evidence_dir/crash-matrix-log.md" "scripts/verify_crash_matrix"
require_text "$evidence_dir/crash-matrix-log.md" "target/crash_matrix/crash_matrix_report.md"

require_text "$feature_dir/final_review.md" "Verdict: PASS"
require_text "$feature_dir/final_review.md" "Non-Visual Evidence: not-applicable"
require_text "$feature_dir/final_review.md" "gate-v1-transactions-wal-recovery"

for implementation_evidence in \
  "$evidence_dir/current-repo-sha.txt" \
  "$evidence_dir/command-log.md" \
  "$evidence_dir/requirement-evidence.md" \
  "$evidence_dir/wal-sidecar-smoke.md" \
  "$evidence_dir/crash-matrix-log.md" \
  "$feature_dir/final_review.md"; do
  reject_scheduler_identity_values "$implementation_evidence"
done

echo "current-artifact WAL evidence contract shape ok"
