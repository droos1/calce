#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:3000}"
DURATION="${DURATION:-10s}"
THREADS="${THREADS:-4}"
CONNECTIONS="${CONNECTIONS:-50}"

echo "=== Calce API Benchmark ==="
echo "URL:         $BASE_URL"
echo "Duration:    $DURATION"
echo "Threads:     $THREADS"
echo "Connections: $CONNECTIONS"
echo ""

# Smoke test
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -H "X-User-Id: alice" \
  "$BASE_URL/v1/users/alice/market-value?as_of_date=2025-03-15&base_currency=SEK")
if [ "$STATUS" != "200" ]; then
  echo "ERROR: API not reachable (status=$STATUS). Is the server running?"
  exit 1
fi

echo "--- Market Value endpoint ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -H "X-User-Id: alice" \
  "$BASE_URL/v1/users/alice/market-value?as_of_date=2025-03-15&base_currency=SEK"

echo ""
echo "--- Portfolio Report endpoint ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -H "X-User-Id: alice" \
  "$BASE_URL/v1/users/alice/portfolio?as_of_date=2025-03-15&base_currency=SEK"

echo ""
echo "--- Auth rejected (403) ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -H "X-User-Id: bob" \
  "$BASE_URL/v1/users/alice/market-value?as_of_date=2025-03-15&base_currency=SEK"
