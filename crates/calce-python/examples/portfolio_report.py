"""Generate a full portfolio report with value changes and allocations."""

from datetime import date

import calce


def main():
    usd = calce.Currency("USD")
    sek = calce.Currency("SEK")
    today = date(2025, 3, 15)

    md = calce.MarketData()

    # Historical prices for AAPL and SPY
    prices = {
        date(2024, 3, 15): {"AAPL": 160.0, "SPY": 450.0},
        date(2024, 12, 31): {"AAPL": 180.0, "SPY": 470.0},
        date(2025, 3, 8): {"AAPL": 190.0, "SPY": 490.0},
        date(2025, 3, 14): {"AAPL": 198.0, "SPY": 498.0},
        today: {"AAPL": 200.0, "SPY": 500.0},
    }
    for d, instruments in prices.items():
        for instrument_id, price in instruments.items():
            md.add_price(instrument_id, d, price)
        md.add_fx_rate(usd, sek, 10.0, d)

    # Instrument metadata
    md.add_instrument_type("AAPL", "stock")
    md.add_instrument_type("SPY", "etf")
    md.add_allocation("AAPL", "sector", "Information Technology", 1.0)
    md.add_allocation("SPY", "sector", "Information Technology", 0.6)
    md.add_allocation("SPY", "sector", "Health Care", 0.4)

    # Trades
    ud = calce.UserData()
    ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 150.0, usd, date(2024, 1, 1)))
    ud.add_trade(calce.Trade("alice", 1, "SPY", 10.0, 440.0, usd, date(2024, 1, 1)))

    engine = calce.CalcEngine(sek, today, "alice", md, ud)
    report = engine.portfolio_report()

    # Market value
    print(f"Market value: {report.market_value.total}\n")

    # Value changes
    vc = report.value_changes
    for label, change in [("Daily", vc.daily), ("Weekly", vc.weekly), ("YTD", vc.ytd), ("Yearly", vc.yearly)]:
        pct = f"{change.change_pct:+.2%}" if change.change_pct is not None else "n/a"
        print(f"  {label:8s} {change.change}  ({pct})")

    # Type allocation
    print(f"\nType allocation:")
    for entry in report.type_allocation.entries:
        print(f"  {entry.instrument_type:8s} {entry.market_value}  ({entry.weight:.0%})")

    # Sector allocation (with fund look-through)
    print(f"\nSector allocation:")
    for entry in report.sector_allocation.entries:
        print(f"  {entry.key:25s} {entry.market_value}  ({entry.weight:.0%})")


if __name__ == "__main__":
    main()
