from datetime import date

import calce


class TestCurrency:
    def test_valid_code(self):
        usd = calce.Currency("USD")
        assert usd.code == "USD"
        assert str(usd) == "USD"

    def test_invalid_code_raises(self):
        try:
            calce.Currency("usd")
            assert False, "Should have raised"
        except ValueError:
            pass

    def test_invalid_length_raises(self):
        try:
            calce.Currency("US")
            assert False, "Should have raised"
        except ValueError:
            pass

    def test_equality(self):
        assert calce.Currency("USD") == calce.Currency("USD")
        assert calce.Currency("USD") != calce.Currency("EUR")

    def test_hashable(self):
        currencies = {calce.Currency("USD"), calce.Currency("USD"), calce.Currency("EUR")}
        assert len(currencies) == 2

    def test_repr(self):
        assert repr(calce.Currency("SEK")) == 'Currency("SEK")'


class TestMoney:
    def test_construction(self):
        usd = calce.Currency("USD")
        m = calce.Money(100.0, usd)
        assert m.amount == 100.0
        assert m.currency == usd

    def test_repr(self):
        m = calce.Money(42.5, calce.Currency("SEK"))
        assert "42.5" in repr(m)
        assert "SEK" in repr(m)

    def test_str(self):
        m = calce.Money(100.0, calce.Currency("USD"))
        assert str(m) == "100 USD"


class TestTrade:
    def test_construction_and_getters(self):
        usd = calce.Currency("USD")
        t = calce.Trade(
            user_id="alice",
            account_id="isa",
            instrument_id="AAPL",
            quantity=100.0,
            price=145.0,
            currency=usd,
            date=date(2025, 1, 15),
        )
        assert t.user_id == "alice"
        assert t.account_id == "isa"
        assert t.instrument_id == "AAPL"
        assert t.quantity == 100.0
        assert t.price == 145.0
        assert t.currency == usd
        assert t.date == date(2025, 1, 15)

    def test_negative_quantity_for_sell(self):
        t = calce.Trade("alice", "isa", "AAPL", -20.0, 155.0, calce.Currency("USD"), date(2025, 1, 15))
        assert t.quantity == -20.0
