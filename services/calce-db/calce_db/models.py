from sqlalchemy import (
    CHAR,
    BigInteger,
    CheckConstraint,
    Column,
    Date,
    DateTime,
    Float,
    ForeignKey,
    Identity,
    Index,
    String,
    UniqueConstraint,
    func,
)
from sqlalchemy.orm import DeclarativeBase, relationship


class Base(DeclarativeBase):
    pass


class Organization(Base):
    __tablename__ = "organizations"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    external_id = Column(String(64), unique=True, nullable=False)
    name = Column(String(200))
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    users = relationship("User", back_populates="organization")


class User(Base):
    __tablename__ = "users"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    external_id = Column(String(64), unique=True, nullable=False)
    email = Column(String(255))
    name = Column(String(200))
    organization_id = Column(BigInteger, ForeignKey("organizations.id"))
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    organization = relationship("Organization", back_populates="users")
    accounts = relationship("Account", back_populates="user")
    trades = relationship("Trade", back_populates="user")


class Instrument(Base):
    __tablename__ = "instruments"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    ticker = Column(String(30), unique=True, nullable=False)
    isin = Column(String(12), unique=True)
    name = Column(String(200))
    instrument_type = Column(String(30), nullable=False, server_default="other")
    currency = Column(CHAR(3), nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())


class Account(Base):
    __tablename__ = "accounts"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    user_id = Column(BigInteger, ForeignKey("users.id"), nullable=False)
    currency = Column(CHAR(3), nullable=False)
    label = Column(String(200), nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    user = relationship("User", back_populates="accounts")
    trades = relationship("Trade", back_populates="account")

    __table_args__ = (
        Index("idx_accounts_user", "user_id"),
        UniqueConstraint("user_id", "label", name="uq_accounts_user_label"),
    )


class Trade(Base):
    __tablename__ = "trades"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    user_id = Column(BigInteger, ForeignKey("users.id"), nullable=False)
    account_id = Column(BigInteger, ForeignKey("accounts.id"), nullable=False)
    instrument_id = Column(BigInteger, ForeignKey("instruments.id"), nullable=False)
    quantity = Column(Float, nullable=False)
    price = Column(Float, nullable=False)
    currency = Column(CHAR(3), nullable=False)
    trade_date = Column(Date, nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    user = relationship("User", back_populates="trades")
    account = relationship("Account", back_populates="trades")
    instrument = relationship("Instrument")

    __table_args__ = (
        CheckConstraint("price >= 0", name="trades_price_check"),
        CheckConstraint("quantity != 0", name="trades_quantity_check"),
        Index("idx_trades_user", "user_id"),
        Index("idx_trades_account", "account_id"),
    )


class Price(Base):
    __tablename__ = "prices"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    instrument_id = Column(BigInteger, ForeignKey("instruments.id"), nullable=False)
    price_date = Column(Date, nullable=False)
    price = Column(Float, nullable=False)

    __table_args__ = (
        UniqueConstraint("instrument_id", "price_date", name="uq_prices_instrument_date"),
        CheckConstraint("price >= 0", name="prices_price_check"),
    )


class FxRate(Base):
    __tablename__ = "fx_rates"

    from_currency = Column(CHAR(3), primary_key=True, nullable=False)
    to_currency = Column(CHAR(3), primary_key=True, nullable=False)
    rate_date = Column(Date, primary_key=True, nullable=False)
    rate = Column(Float, nullable=False)

    __table_args__ = (CheckConstraint("rate > 0", name="fx_rates_rate_check"),)
