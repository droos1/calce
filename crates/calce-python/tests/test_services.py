from datetime import date

import calce


class TestMarketData:
    def test_create_empty(self):
        md = calce.MarketData()
        assert md is not None

    def test_add_price(self):
        md = calce.MarketData()
        md.add_price("AAPL", date(2025, 1, 15), 150.0)

    def test_add_fx_rate(self):
        md = calce.MarketData()
        usd = calce.Currency("USD")
        sek = calce.Currency("SEK")
        md.add_fx_rate(usd, sek, 10.5, date(2025, 1, 15))

    def test_add_instrument_type(self):
        md = calce.MarketData()
        md.add_instrument_type("AAPL", "stock")
        md.add_instrument_type("SPY", "etf")
        md.add_instrument_type("UNKNOWN", "some_random_thing")


class TestUserData:
    def test_create_empty(self):
        ud = calce.UserData()
        assert ud is not None

    def test_add_trade(self):
        ud = calce.UserData()
        t = calce.Trade("alice", 1, "AAPL", 100.0, 145.0, calce.Currency("USD"), date(2025, 1, 15))
        ud.add_trade(t)
