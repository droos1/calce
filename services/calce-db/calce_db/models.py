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
    Integer,
    String,
    UniqueConstraint,
    Uuid,
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
    role = Column(String(20), nullable=False, server_default="user")
    organization_id = Column(BigInteger, ForeignKey("organizations.id"))
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    organization = relationship("Organization", back_populates="users")
    accounts = relationship("Account", back_populates="user")
    trades = relationship("Trade", back_populates="user")
    credential = relationship("UserCredential", back_populates="user", uselist=False)


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


class UserCredential(Base):
    __tablename__ = "user_credentials"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    user_id = Column(BigInteger, ForeignKey("users.id", ondelete="CASCADE"), unique=True, nullable=False)
    password_hash = Column(String(255), nullable=False)
    failed_attempts = Column(Integer, nullable=False, server_default="0")
    locked_until = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())
    updated_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    user = relationship("User", back_populates="credential")


class RefreshToken(Base):
    __tablename__ = "refresh_tokens"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    user_id = Column(BigInteger, ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    family_id = Column(Uuid, nullable=False)
    token_hash = Column(String(128), unique=True, nullable=False)
    superseded_at = Column(DateTime(timezone=True))
    revoked_at = Column(DateTime(timezone=True))
    expires_at = Column(DateTime(timezone=True), nullable=False)
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    user = relationship("User")

    __table_args__ = (
        Index("idx_refresh_tokens_family", "family_id"),
        Index("idx_refresh_tokens_user", "user_id"),
    )


class ApiKey(Base):
    __tablename__ = "api_keys"

    id = Column(BigInteger, Identity(always=True), primary_key=True)
    organization_id = Column(BigInteger, ForeignKey("organizations.id", ondelete="CASCADE"), nullable=False)
    name = Column(String(100), nullable=False)
    key_prefix = Column(String(20), nullable=False)
    key_hash = Column(String(128), unique=True, nullable=False)
    expires_at = Column(DateTime(timezone=True))
    revoked_at = Column(DateTime(timezone=True))
    created_at = Column(DateTime(timezone=True), nullable=False, server_default=func.now())

    organization = relationship("Organization")

    __table_args__ = (Index("idx_api_keys_organization", "organization_id"),)
