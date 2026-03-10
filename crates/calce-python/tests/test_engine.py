"""Port of the Rust multi-currency integration test to Python."""

from datetime import date

import calce


def setup_multi_currency():
    """Mirror of setup_multi_currency_scenario from Rust integration tests."""
    usd = calce.Currency("USD")
    eur = calce.Currency("EUR")
    sek = calce.Currency("SEK")
    d = date(2025, 1, 15)

    md = calce.MarketData()
    md.add_price("AAPL", d, 150.0)
    md.add_price("VOW3", d, 120.0)
    md.add_fx_rate(usd, sek, 10.5, d)
    md.add_fx_rate(eur, sek, 11.4, d)

    ud = calce.UserData()
    # Alice buys 100 AAPL, sells 20 → net 80
    ud.add_trade(calce.Trade("alice", "alice-usd", "AAPL", 100.0, 145.0, usd, d))
    ud.add_trade(calce.Trade("alice", "alice-usd", "AAPL", -20.0, 155.0, usd, d))
    # Alice buys 50 VOW3
    ud.add_trade(calce.Trade("alice", "alice-eur", "VOW3", 50.0, 115.0, eur, d))

    return md, ud, sek, d


class TestMarketValue:
    def test_multi_currency_portfolio(self):
        md, ud, sek, d = setup_multi_currency()
        engine = calce.CalcEngine(sek, d, "alice", md, ud)
        result = engine.market_value()

        # AAPL: 80 * 150 = 12,000 USD → 12,000 * 10.5 = 126,000 SEK
        # VOW3: 50 * 120 = 6,000 EUR → 6,000 * 11.4 = 68,400 SEK
        # Total: 126,000 + 68,400 = 194,400 SEK
        assert len(result.positions) == 2
        assert result.total.amount == 194_400.0
        assert result.total.currency == sek

        aapl = result.positions[0]
        assert aapl.instrument_id == "AAPL"
        assert aapl.quantity == 80.0
        assert aapl.market_value.amount == 12_000.0
        assert aapl.market_value_base.amount == 126_000.0

        vow3 = result.positions[1]
        assert vow3.instrument_id == "VOW3"
        assert vow3.quantity == 50.0
        assert vow3.market_value.amount == 6_000.0
        assert vow3.market_value_base.amount == 68_400.0

    def test_same_currency_no_fx(self):
        usd = calce.Currency("USD")
        d = date(2025, 1, 15)

        md = calce.MarketData()
        md.add_price("AAPL", d, 150.0)

        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", "acct", "AAPL", 100.0, 145.0, usd, d))

        engine = calce.CalcEngine(usd, d, "alice", md, ud)
        result = engine.market_value()

        assert result.total.amount == 15_000.0
        assert result.total.currency == usd


class TestPortfolioReport:
    def test_full_report(self):
        usd = calce.Currency("USD")
        sek = calce.Currency("SEK")
        today = date(2025, 3, 15)
        trade_date = date(2024, 1, 1)

        md = calce.MarketData()
        md.add_price("AAPL", today, 200.0)
        md.add_price("AAPL", date(2025, 3, 14), 198.0)
        md.add_price("AAPL", date(2025, 3, 8), 190.0)
        md.add_price("AAPL", date(2024, 3, 15), 160.0)
        md.add_price("AAPL", date(2024, 12, 31), 180.0)
        for d in [today, date(2025, 3, 14), date(2025, 3, 8), date(2024, 3, 15), date(2024, 12, 31)]:
            md.add_fx_rate(usd, sek, 10.0, d)

        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", "acct", "AAPL", 100.0, 150.0, usd, trade_date))

        engine = calce.CalcEngine(sek, today, "alice", md, ud)
        report = engine.portfolio_report()

        # Market value: 100 * 200 * 10 = 200,000 SEK
        assert report.market_value.total.amount == 200_000.0
        assert len(report.market_value.positions) == 1

        # Value changes
        assert report.value_changes.market_value.amount == 200_000.0
        assert report.value_changes.daily.change.amount == 2_000.0
        assert report.value_changes.weekly.change.amount == 10_000.0
        assert report.value_changes.yearly.change.amount == 40_000.0
        assert report.value_changes.ytd.change.amount == 20_000.0

        # Percentage checks
        assert report.value_changes.daily.change_pct is not None
        assert report.value_changes.yearly.change_pct is not None

