#!/usr/bin/env zsh
# test-report.sh — run the full NyxForge test suite and email a report.
#
# Usage:
#   ./scripts/test-report.sh           # run tests + email report
#   ./scripts/test-report.sh --no-mail # run tests, print report, skip email
#
# nyxforge-web targets wasm32 and requires wasm-pack — excluded from this run.
# Requires: cargo (rustup), msmtp (~/.msmtprc configured).

set -uo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$HOME/Library/Logs/nyxforge-tests"
TIMESTAMP="$(date '+%Y-%m-%d %H:%M:%S %Z')"
DATE_SLUG="$(date '+%Y%m%d-%H%M%S')"
RAW_LOG="$LOG_DIR/raw-${DATE_SLUG}.log"
TO="archerships@protonmail.ch"
NO_MAIL="${1:-}"

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

mkdir -p "$LOG_DIR"
cd "$REPO_DIR"

# ---------------------------------------------------------------------------
# 1. Run tests, capture output
# ---------------------------------------------------------------------------

cargo test --workspace --exclude nyxforge-web --color never 2>&1 | tee "$RAW_LOG"
TEST_EXIT=$pipestatus[1]   # zsh pipestatus is 1-indexed, lowercase

# ---------------------------------------------------------------------------
# 2. Parse results with awk (robust on macOS)
# ---------------------------------------------------------------------------

# "test result: ok. 9 passed; 0 failed; 0 ignored; ..."
# Fields: $5=passed_count $6="passed;" $7=failed_count $8="failed;" etc.
read -r total_pass total_fail total_ignored <<< "$(awk '
/test result:/ {
    for (i = 1; i <= NF; i++) {
        if ($i == "passed;")  total_pass  += $(i-1)
        if ($i == "failed;")  total_fail  += $(i-1)
        if ($i == "ignored;") total_ignored += $(i-1)
    }
}
END { print total_pass+0, total_fail+0, total_ignored+0 }
' "$RAW_LOG")"

STATUS=$([[ $TEST_EXIT -eq 0 ]] && echo "PASS" || echo "FAIL")

# One FAILED test name per line
failed_list="$(awk '/^test .* \.\.\. (FAILED|failed)/{print "  " $0}' "$RAW_LOG")"

# Per-crate summary: "Running unittests …" followed by its "test result:" line
crate_summary="$(awk '
    /Running unittests|Running tests/ { crate = $0; next }
    /test result:/                    { if (crate != "") { print crate; print "  " $0; print ""; crate = "" } }
' "$RAW_LOG")"

# ---------------------------------------------------------------------------
# 3. Build report
# ---------------------------------------------------------------------------

REPORT="NyxForge Test Report
====================
Date    : ${TIMESTAMP}
Status  : ${STATUS}
Repo    : ${REPO_DIR}

Summary
-------
Passed  : ${total_pass}
Failed  : ${total_fail}
Ignored : ${total_ignored}
Total   : $((total_pass + total_fail))

Per-Crate Results
-----------------
${crate_summary}"

if [[ -n "$failed_list" ]]; then
    REPORT="${REPORT}
Failed Tests
------------
${failed_list}"
fi

REPORT="${REPORT}

Full log: ${RAW_LOG}"

# ---------------------------------------------------------------------------
# 4. Print to stdout
# ---------------------------------------------------------------------------

printf '%s\n' "$REPORT"

# ---------------------------------------------------------------------------
# 5. Email report
# ---------------------------------------------------------------------------

if [[ "$NO_MAIL" == "--no-mail" ]]; then
    echo "(mail skipped — --no-mail flag)"
    exit $TEST_EXIT
fi

SUBJECT="[NyxForge] Tests ${STATUS}: ${total_pass} passed, ${total_fail} failed — ${TIMESTAMP}"

{
    printf 'To: %s\n'                                    "$TO"
    printf 'Subject: %s\n'                               "$SUBJECT"
    printf 'Content-Type: text/plain; charset=utf-8\n'
    printf '\n'
    printf '%s\n'                                        "$REPORT"
} | msmtp "$TO"

echo "Report mailed to $TO"
exit $TEST_EXIT
