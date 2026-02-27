from datetime import date

import calce


class TestExceptionHierarchy:
    def test_all_exceptions_inherit_from_calce_error(self):
        assert issubclass(calce.UnauthorizedError, calce.CalceError)
        assert issubclass(calce.PriceNotFoundError, calce.CalceError)
        assert issubclass(calce.FxRateNotFoundError, calce.CalceError)
        assert issubclass(calce.NoTradesFoundError, calce.CalceError)
        assert issubclass(calce.CurrencyMismatchError, calce.CalceError)

    def test_calce_error_inherits_from_exception(self):
        assert issubclass(calce.CalceError, Exception)


class TestMissingPrice:
    def test_missing_price_raises(self):
        usd = calce.Currency("USD")
        d = date(2025, 1, 15)

        md = calce.MarketData()
        # No price added for AAPL

        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", "acct", "AAPL", 100.0, 145.0, usd, d))

        engine = calce.CalcEngine(usd, d, "alice", md, ud)
        try:
            engine.market_value()
            assert False, "Should have raised PriceNotFoundError"
        except calce.PriceNotFoundError as e:
            assert "AAPL" in str(e)

    def test_missing_price_caught_as_calce_error(self):
        usd = calce.Currency("USD")
        d = date(2025, 1, 15)

        md = calce.MarketData()
        ud = calce.UserData()
        ud.add_trade(calce.Trade("alice", "acct", "AAPL", 100.0, 145.0, usd, d))

        engine = calce.CalcEngine(usd, d, "alice", md, ud)
        try:
            engine.market_value()
            assert False, "Should have raised"
        except calce.CalceError:
            pass  # Caught via base class


class TestNoTradesFound:
    def test_no_trades_raises(self):
        usd = calce.Currency("USD")
        d = date(2025, 1, 15)

        md = calce.MarketData()
        ud = calce.UserData()

        engine = calce.CalcEngine(usd, d, "alice", md, ud)
        try:
            engine.market_value()
            assert False, "Should have raised NoTradesFoundError"
        except calce.NoTradesFoundError as e:
            assert "alice" in str(e)


class TestInvalidRole:
    def test_invalid_role_raises(self):
        usd = calce.Currency("USD")
        try:
            calce.CalcEngine(usd, date(2025, 1, 15), "alice", calce.MarketData(), calce.UserData(), role="superuser")
            assert False, "Should have raised"
        except ValueError as e:
            assert "superuser" in str(e)
