#!/usr/bin/env bash
# Smoke test the Calce API endpoints against seeded data.
#
# Usage: ./tools/smoke_test.sh [base_url]

set -euo pipefail

PORT="${PORT:-35701}"
BASE="${1:-http://localhost:$PORT}"
AS_OF_DATE="${AS_OF_DATE:-$(date +%Y-%m-%d)}"
PASS=0
FAIL=0
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

millis() {
    python3 -c 'import time; print(int(time.time()*1000))'
}

# check LABEL EXPECTED_STATUS URL [CURL_ARGS...]
# Uses a temp file for the body so status code parsing is clean.
check() {
    local label="$1"
    local expected_status="$2"
    local url="$3"
    shift 3

    local start http_code ms
    start=$(millis)

    # Write body to file, capture only the status code
    if [ $# -gt 0 ]; then
        http_code=$(curl -s -o "$TMPFILE" -w '%{http_code}' "$@" "$url")
    else
        http_code=$(curl -s -o "$TMPFILE" -w '%{http_code}' "$url")
    fi

    ms=$(( $(millis) - start ))

    if [ "$http_code" = "$expected_status" ]; then
        echo "  PASS  ${label} (${http_code}) ${ms}ms"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  ${label} — expected ${expected_status}, got ${http_code} (${ms}ms)"
        if command -v jq &>/dev/null; then
            jq . "$TMPFILE" 2>/dev/null || cat "$TMPFILE"
        else
            cat "$TMPFILE"
        fi
        echo ""
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Calce API Smoke Tests ==="
echo "Base URL: ${BASE}"
echo "As-of date: ${AS_OF_DATE}"
echo ""

# --- Check server is reachable ---
if ! curl -s -o /dev/null --connect-timeout 2 "${BASE}/healthz"; then
    echo "ERROR: Cannot connect to ${BASE}. Is the API server running?"
    echo "  Start it with: cargo run -p calce-api"
    exit 1
fi

# --- Discover test data from DB ---

DB_URL="${DB_URL:-postgres://calce:calce@localhost:5433/calce}"

# Pick first, middle, and last user
mapfile -t ALL_USERS < <(psql "$DB_URL" -tAc "SELECT id FROM users ORDER BY id" 2>/dev/null || true)
if [ ${#ALL_USERS[@]} -eq 0 ]; then
    echo "ERROR: Could not query users from DB. Is it seeded?"
    exit 1
fi
MID=$(( ${#ALL_USERS[@]} / 2 ))
USERS=("${ALL_USERS[0]}" "${ALL_USERS[$MID]}" "${ALL_USERS[-1]}")

INSTRUMENT=$(psql "$DB_URL" -tAc "SELECT id FROM instruments LIMIT 1" 2>/dev/null || echo "")
if [ -z "$INSTRUMENT" ]; then
    echo "ERROR: Could not query instruments from DB. Is it seeded?"
    exit 1
fi

echo "Test users: ${USERS[*]}"
echo "Test instrument: ${INSTRUMENT}"
echo ""

# --- User-scoped endpoints ---

for user in "${USERS[@]}"; do
    echo "--- Market value: ${user} ---"
    check "market-value/${user}/SEK" 200 \
        "${BASE}/v1/users/${user}/market-value?as_of_date=${AS_OF_DATE}&base_currency=SEK" \
        -H "x-user-id: ${user}" -H "x-role: admin"

    echo "--- Portfolio: ${user} ---"
    check "portfolio/${user}/SEK" 200 \
        "${BASE}/v1/users/${user}/portfolio?as_of_date=${AS_OF_DATE}&base_currency=SEK" \
        -H "x-user-id: ${user}" -H "x-role: admin"
done

FIRST_USER="${USERS[0]}"

# Test with different base currencies
echo "--- Market value: ${FIRST_USER}/USD ---"
check "market-value/${FIRST_USER}/USD" 200 \
    "${BASE}/v1/users/${FIRST_USER}/market-value?as_of_date=${AS_OF_DATE}&base_currency=USD" \
    -H "x-user-id: ${FIRST_USER}" -H "x-role: admin"

echo "--- Market value: ${FIRST_USER}/EUR ---"
check "market-value/${FIRST_USER}/EUR" 200 \
    "${BASE}/v1/users/${FIRST_USER}/market-value?as_of_date=${AS_OF_DATE}&base_currency=EUR" \
    -H "x-user-id: ${FIRST_USER}" -H "x-role: admin"

# --- Instrument-scoped endpoints ---

echo ""
echo "--- Volatility (instrument: ${INSTRUMENT}) ---"
check "volatility/365d" 200 \
    "${BASE}/v1/instruments/${INSTRUMENT}/volatility?as_of_date=${AS_OF_DATE}&lookback_days=365" \
    -H "x-user-id: ${FIRST_USER}"

check "volatility/90d" 200 \
    "${BASE}/v1/instruments/${INSTRUMENT}/volatility?as_of_date=${AS_OF_DATE}&lookback_days=90" \
    -H "x-user-id: ${FIRST_USER}"

# --- Error cases ---

echo ""
echo "--- Error cases ---"

check "missing-auth" 401 \
    "${BASE}/v1/users/${FIRST_USER}/market-value?as_of_date=${AS_OF_DATE}&base_currency=SEK"

check "bad-currency" 400 \
    "${BASE}/v1/users/${FIRST_USER}/market-value?as_of_date=${AS_OF_DATE}&base_currency=NOPE" \
    -H "x-user-id: ${FIRST_USER}" -H "x-role: admin"

# Access denied: second user tries to access first user's data
SECOND_USER="${ALL_USERS[1]}"
check "access-denied" 403 \
    "${BASE}/v1/users/${FIRST_USER}/market-value?as_of_date=${AS_OF_DATE}&base_currency=SEK" \
    -H "x-user-id: ${SECOND_USER}" -H "x-role: user"

# --- Summary ---

echo ""
echo "=== Results: ${PASS} passed, ${FAIL} failed ==="
[ "$FAIL" -eq 0 ] || exit 1
