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
    ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 145.0, usd, d))
    ud.add_trade(calce.Trade("alice", 1, "AAPL", -20.0, 155.0, usd, d))
    # Alice buys 50 VOW3
    ud.add_trade(calce.Trade("alice", 2, "VOW3", 50.0, 115.0, eur, d))

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
        ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 145.0, usd, d))

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
        ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 150.0, usd, trade_date))

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

        # Type allocation: no types set, so single entry as "other"
        alloc = report.type_allocation
        assert len(alloc.entries) == 1
        assert alloc.entries[0].instrument_type == "other"
        assert abs(alloc.entries[0].weight - 1.0) < 1e-10

    def test_type_allocation_with_types(self):
        usd = calce.Currency("USD")
        sek = calce.Currency("SEK")
        today = date(2025, 3, 15)
        trade_date = date(2024, 1, 1)

        md = calce.MarketData()
        md.add_price("AAPL", today, 200.0)
        md.add_price("SPY", today, 500.0)
        for d in [today, date(2025, 3, 14), date(2025, 3, 8), date(2024, 3, 15), date(2024, 12, 31)]:
            md.add_price("AAPL", d, 200.0)
            md.add_price("SPY", d, 500.0)
            md.add_fx_rate(usd, sek, 10.0, d)
        md.add_instrument_type("AAPL", "stock")
        md.add_instrument_type("SPY", "etf")

        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 150.0, usd, trade_date))
        ud.add_trade(calce.Trade("alice", 1, "SPY", 10.0, 480.0, usd, trade_date))

        engine = calce.CalcEngine(sek, today, "alice", md, ud)
        report = engine.portfolio_report()

        alloc = report.type_allocation
        assert len(alloc.entries) == 2

        # AAPL: 100 * 200 * 10 = 200,000 (stock)
        # SPY:  10 * 500 * 10 = 50,000 (etf)
        # Total: 250,000
        # Stock weight = 200,000 / 250,000 = 0.8
        # ETF weight = 50,000 / 250,000 = 0.2
        assert alloc.entries[0].instrument_type == "stock"
        assert abs(alloc.entries[0].weight - 0.8) < 1e-10
        assert alloc.entries[1].instrument_type == "etf"
        assert abs(alloc.entries[1].weight - 0.2) < 1e-10

        # Weights sum to 1.0
        total_weight = sum(e.weight for e in alloc.entries)
        assert abs(total_weight - 1.0) < 1e-10

    def test_sector_allocation_with_fund_lookthrough(self):
        usd = calce.Currency("USD")
        sek = calce.Currency("SEK")
        today = date(2025, 3, 15)
        trade_date = date(2024, 1, 1)

        md = calce.MarketData()
        md.add_price("AAPL", today, 200.0)
        md.add_price("SPY", today, 500.0)
        for d in [today, date(2025, 3, 14), date(2025, 3, 8), date(2024, 3, 15), date(2024, 12, 31)]:
            md.add_price("AAPL", d, 200.0)
            md.add_price("SPY", d, 500.0)
            md.add_fx_rate(usd, sek, 10.0, d)
        md.add_instrument_type("AAPL", "stock")
        md.add_instrument_type("SPY", "etf")
        # AAPL: 100% Info Tech
        md.add_allocation("AAPL", "sector", "Information Technology", 1.0)
        # SPY: multi-sector
        md.add_allocation("SPY", "sector", "Information Technology", 0.6)
        md.add_allocation("SPY", "sector", "Health Care", 0.4)

        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", 1, "AAPL", 100.0, 150.0, usd, trade_date))
        ud.add_trade(calce.Trade("alice", 1, "SPY", 10.0, 480.0, usd, trade_date))

        engine = calce.CalcEngine(sek, today, "alice", md, ud)
        report = engine.portfolio_report()

        alloc = report.sector_allocation
        assert alloc.dimension == "sector"

        # AAPL: 100 * 200 * 10 = 200,000 SEK → 100% Info Tech = 200,000
        # SPY:  10 * 500 * 10 = 50,000 SEK → 60% Info Tech = 30,000, 40% Health Care = 20,000
        # Info Tech total: 200,000 + 30,000 = 230,000 / 250,000 = 0.92
        # Health Care total: 20,000 / 250,000 = 0.08
        entries = {e.key: e for e in alloc.entries}
        assert "Information Technology" in entries
        assert abs(entries["Information Technology"].market_value.amount - 230_000.0) < 1e-6
        assert abs(entries["Information Technology"].weight - 0.92) < 1e-6
        assert "Health Care" in entries
        assert abs(entries["Health Care"].market_value.amount - 20_000.0) < 1e-6
        assert abs(entries["Health Care"].weight - 0.08) < 1e-6
