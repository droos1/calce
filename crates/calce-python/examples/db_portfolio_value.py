"""Load market data from the database and calculate portfolio value.

Requires a running calce database (invoke dev).
"""

import os
from datetime import date, timedelta

import calce

DB_URL = os.environ.get("DATABASE_URL", "postgresql://calce:calce@localhost:5433/calce")


def main():
    ds = calce.DataService(DB_URL)

    stats = ds.data_stats()
    print(f"Loaded {stats.instrument_count} instruments, {stats.price_count} prices, {stats.fx_rate_count} FX rates\n")

    # Pick the first user that has trades
    users = ds.list_users()
    if not users:
        print("No users found in database.")
        return

    user = users[0]
    print(f"User: {user.id} ({user.trade_count} trades)\n")

    # Use yesterday to avoid missing prices for today
    as_of = date.today() - timedelta(days=1)
    engine = ds.engine(user.id, "SEK", as_of)
    result = engine.market_value()

    print(f"Portfolio value: {result.total}")
    for pos in result.positions:
        print(f"  {pos.instrument_id:10s} {pos.quantity:>8.0f} × {pos.price}  =  {pos.market_value_base}")

    for w in result.warnings:
        print(f"  ⚠ {w.code}: {w.message}")


if __name__ == "__main__":
    main()
