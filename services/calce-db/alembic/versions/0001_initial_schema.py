"""initial schema

Revision ID: 0001
Revises:
Create Date: 2026-03-17

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0001"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "users",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column("external_id", sa.String(64), unique=True, nullable=False),
        sa.Column("email", sa.String(255)),
        sa.Column("name", sa.String(200)),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
    )

    op.create_table(
        "instruments",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column("ticker", sa.String(30), unique=True, nullable=False),
        sa.Column("isin", sa.String(12), unique=True),
        sa.Column("name", sa.String(200)),
        sa.Column(
            "instrument_type", sa.String(30), nullable=False, server_default="other"
        ),
        sa.Column("currency", sa.CHAR(3), nullable=False),
    )

    op.create_table(
        "accounts",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column(
            "user_id",
            sa.BigInteger,
            sa.ForeignKey("users.id"),
            nullable=False,
        ),
        sa.Column("currency", sa.CHAR(3), nullable=False),
        sa.Column("label", sa.String(200), nullable=False),
    )
    op.create_index("idx_accounts_user", "accounts", ["user_id"])

    op.create_table(
        "trades",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column(
            "user_id",
            sa.BigInteger,
            sa.ForeignKey("users.id"),
            nullable=False,
        ),
        sa.Column(
            "account_id",
            sa.BigInteger,
            sa.ForeignKey("accounts.id"),
            nullable=False,
        ),
        sa.Column(
            "instrument_id",
            sa.BigInteger,
            sa.ForeignKey("instruments.id"),
            nullable=False,
        ),
        sa.Column("quantity", sa.Float, nullable=False),
        sa.Column("price", sa.Float, nullable=False),
        sa.Column("currency", sa.CHAR(3), nullable=False),
        sa.Column("trade_date", sa.Date, nullable=False),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.CheckConstraint("price >= 0", name="trades_price_check"),
    )
    op.create_index("idx_trades_user", "trades", ["user_id"])
    op.create_index(
        "idx_trades_instrument_date", "trades", ["instrument_id", "trade_date"]
    )

    op.create_table(
        "prices",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column(
            "instrument_id",
            sa.BigInteger,
            sa.ForeignKey("instruments.id"),
            nullable=False,
        ),
        sa.Column("price_date", sa.Date, nullable=False),
        sa.Column("price", sa.Float, nullable=False),
        sa.UniqueConstraint(
            "instrument_id", "price_date", name="uq_prices_instrument_date"
        ),
        sa.CheckConstraint("price >= 0", name="prices_price_check"),
    )

    op.create_table(
        "fx_rates",
        sa.Column("from_currency", sa.CHAR(3), nullable=False),
        sa.Column("to_currency", sa.CHAR(3), nullable=False),
        sa.Column("rate_date", sa.Date, nullable=False),
        sa.Column("rate", sa.Float, nullable=False),
        sa.PrimaryKeyConstraint("from_currency", "to_currency", "rate_date"),
        sa.CheckConstraint("rate > 0", name="fx_rates_rate_check"),
    )
    op.create_index("idx_fx_rates_date", "fx_rates", ["rate_date"])

    # Drop the old sqlx migration tracking table
    op.execute("DROP TABLE IF EXISTS _sqlx_migrations")


def downgrade() -> None:
    op.drop_table("fx_rates")
    op.drop_table("prices")
    op.drop_table("trades")
    op.drop_table("accounts")
    op.drop_table("instruments")
    op.drop_table("users")
