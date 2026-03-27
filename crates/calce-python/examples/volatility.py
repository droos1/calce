"""Calculate historical volatility for an instrument."""

from datetime import date, timedelta

import calce


def main():
    usd = calce.Currency("USD")
    base_date = date(2025, 3, 15)

    md = calce.MarketData()

    # Simulate 90 days of price data with some movement
    price = 150.0
    import math

    for i in range(90):
        d = base_date - timedelta(days=89 - i)
        # Sine wave + upward drift to create realistic-ish price movement
        price = 150.0 + i * 0.2 + 8.0 * math.sin(i * 0.3)
        md.add_price("AAPL", d, round(price, 2))

    # Need a trade so the engine has something to work with
    ud = calce.UserData()
    ud.add_trade(calce.Trade("alice", 1, "AAPL", 10.0, 150.0, usd, base_date - timedelta(days=89)))

    engine = calce.CalcEngine(usd, base_date, "alice", md, ud)
    vol = engine.volatility("AAPL", lookback_days=90)

    print(f"AAPL volatility ({vol.start_date} to {vol.end_date})")
    print(f"  Daily:      {vol.daily_volatility:.4f}")
    print(f"  Annualized: {vol.annualized_volatility:.2%}")
    print(f"  Observations: {vol.num_observations}")


if __name__ == "__main__":
    main()
