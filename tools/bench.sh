#!/usr/bin/env bash
set -euo pipefail

PORT="${PORT:-35701}"
BASE_URL="${BASE_URL:-http://localhost:$PORT}"
DURATION="${DURATION:-10s}"
THREADS="${THREADS:-4}"
CONNECTIONS="${CONNECTIONS:-50}"

echo "=== Calce API Benchmark ==="
echo "URL:         $BASE_URL"
echo "Duration:    $DURATION"
echo "Threads:     $THREADS"
echo "Connections: $CONNECTIONS"
echo ""

# Login to get a JWT token
echo "Authenticating..."
LOGIN_RESP=$(curl -sf -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@njorda.se","password":"protectme"}' 2>/dev/null) || {
  echo "ERROR: Cannot login. Is the server running on port ${PORT}?"
  echo "  Start with: invoke rel"
  exit 1
}
TOKEN=$(echo "$LOGIN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")
AUTH="Authorization: Bearer $TOKEN"
echo "Authenticated."
echo ""

# Pick a user with trades (admin has none, so query for one)
USER_ID=$(curl -sf -H "$AUTH" "$BASE_URL/v1/data/users?limit=1" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['items'][0]['id'])")
# Find an instrument that has price data for the volatility benchmark.
# Fetch a batch and pick the first one where the volatility endpoint returns 200.
INSTRUMENT=""
TICKERS=$(curl -sf -H "$AUTH" "$BASE_URL/v1/data/instruments?limit=20" \
  | python3 -c "import sys,json; [print(i['ticker']) for i in json.load(sys.stdin)['items']]")
for t in $TICKERS; do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" -H "$AUTH" \
    "$BASE_URL/v1/instruments/$t/volatility?as_of_date=2025-03-14&lookback_days=365")
  if [ "$CODE" = "200" ]; then
    INSTRUMENT="$t"
    break
  fi
done
if [ -z "$INSTRUMENT" ]; then
  echo "WARNING: No instrument with price data found, skipping volatility benchmark"
fi

echo "Test user:       $USER_ID"
echo "Test instrument: $INSTRUMENT"
echo ""

echo "--- Market Value endpoint ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -H "$AUTH" \
  "$BASE_URL/v1/users/$USER_ID/market-value?as_of_date=2025-03-14&base_currency=SEK"

echo ""
echo "--- Portfolio Report endpoint ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -H "$AUTH" \
  "$BASE_URL/v1/users/$USER_ID/portfolio?as_of_date=2025-03-14&base_currency=SEK"

echo ""
if [ -n "$INSTRUMENT" ]; then
  echo "--- Volatility endpoint ---"
  wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
    -H "$AUTH" \
    "$BASE_URL/v1/instruments/$INSTRUMENT/volatility?as_of_date=2025-03-14&lookback_days=365"
else
  echo "--- Volatility endpoint: SKIPPED (no instrument with price data) ---"
fi

echo ""
echo "--- Auth rejected (401, no token) ---"
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  "$BASE_URL/v1/users/$USER_ID/market-value?as_of_date=2025-03-14&base_currency=SEK"
