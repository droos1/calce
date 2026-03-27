"""Search instruments and fetch price history from the database.

Requires a running calce database (invoke dev).
"""

import os
from datetime import date, timedelta

import calce

DB_URL = os.environ.get("DATABASE_URL", "postgresql://calce:calce@localhost:5433/calce")


def main():
    ds = calce.DataService(DB_URL)

    # Search for instruments
    query = "stock"
    results = ds.search_instruments(query)
    print(f"Search '{query}': {len(results)} results")
    for inst in results[:5]:
        print(f"  {inst.id:10s} {inst.currency:5s} {inst.instrument_type:8s} {inst.name or ''}")

    if not results:
        return

    # Get 30-day price history for the first match that has data
    today = date.today()
    for instrument in results[:10]:
        try:
            history = ds.get_price_history(instrument.id, today - timedelta(days=30), today)
        except calce.PriceNotFoundError:
            continue
        print(f"\n{instrument.id} — last {len(history)} prices:")
        for point in history[-10:]:  # show last 10
            print(f"  {point.date}  {point.price:>10.2f}")
        break


if __name__ == "__main__":
    main()
