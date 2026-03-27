"""Full portfolio report from the database — value changes, type and sector allocation.

Requires a running calce database (invoke dev).
"""

import os
from datetime import date, timedelta

import calce

DB_URL = os.environ.get("DATABASE_URL", "postgresql://calce:calce@localhost:5433/calce")


def main():
    ds = calce.DataService(DB_URL)

    users = ds.list_users()
    if not users:
        print("No users found in database.")
        return

    user = users[0]
    # Use yesterday to avoid missing prices for today
    as_of = date.today() - timedelta(days=1)
    engine = ds.engine(user.id, "SEK", as_of)
    report = engine.portfolio_report()

    # Market value
    print(f"Portfolio report for {user.id}")
    print(f"Market value: {report.market_value.total}\n")

    # Value changes
    vc = report.value_changes
    print("Value changes:")
    for label, change in [("Daily", vc.daily), ("Weekly", vc.weekly), ("YTD", vc.ytd), ("Yearly", vc.yearly)]:
        pct = f"{change.change_pct:+.2%}" if change.change_pct is not None else "n/a"
        print(f"  {label:8s} {change.change}  ({pct})")

    # Type allocation
    if report.type_allocation.entries:
        print(f"\nType allocation:")
        for entry in report.type_allocation.entries:
            print(f"  {entry.instrument_type:10s} {entry.market_value}  ({entry.weight:.0%})")

    # Sector allocation
    if report.sector_allocation.entries:
        print(f"\nSector allocation:")
        for entry in report.sector_allocation.entries:
            print(f"  {entry.key:25s} {entry.market_value}  ({entry.weight:.0%})")


if __name__ == "__main__":
    main()
