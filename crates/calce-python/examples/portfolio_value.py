"""Calculate the market value of a multi-currency portfolio."""

from datetime import date

import calce


def main():
    usd = calce.Currency("USD")
    eur = calce.Currency("EUR")
    sek = calce.Currency("SEK")
    today = date(2025, 3, 15)

    # Set up market data
    md = calce.MarketData()
    md.add_price("AAPL", today, 200.0)
    md.add_price("VOW3", today, 120.0)
    md.add_fx_rate(usd, sek, 10.5, today)
    md.add_fx_rate(eur, sek, 11.4, today)

    # Record trades
    ud = calce.UserData()
    ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 195.0, usd, today))
    ud.add_trade(calce.Trade("alice", 1, "AAPL", -20.0, 205.0, usd, today))  # partial sell
    ud.add_trade(calce.Trade("alice", 2, "VOW3", 50.0, 115.0, eur, today))

    # Calculate market value in SEK
    engine = calce.CalcEngine(sek, today, "alice", md, ud)
    result = engine.market_value()

    print(f"Portfolio value: {result.total}")
    for pos in result.positions:
        print(f"  {pos.instrument_id}: {pos.quantity:.0f} shares @ {pos.price} = {pos.market_value} ({pos.market_value_base})")

    for w in result.warnings:
        print(f"  ⚠ {w.code}: {w.message}")


if __name__ == "__main__":
    main()
