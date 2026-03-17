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

_TIMESTAMP_COLS = [
    sa.Column(
        "created_at",
        sa.DateTime(timezone=True),
        nullable=False,
        server_default=sa.func.now(),
    ),
    sa.Column(
        "updated_at",
        sa.DateTime(timezone=True),
        nullable=False,
        server_default=sa.func.now(),
    ),
]

# Tables that have updated_at managed by trigger
_UPDATED_AT_TABLES = ["users", "instruments", "accounts"]


def upgrade() -> None:
    # -- Trigger function (shared by all tables with updated_at) --
    op.execute(
        """
        CREATE OR REPLACE FUNCTION set_updated_at()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = now();
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql
        """
    )

    op.create_table(
        "users",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column("external_id", sa.String(64), unique=True, nullable=False),
        sa.Column("email", sa.String(255)),
        sa.Column("name", sa.String(200)),
        *_TIMESTAMP_COLS,
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
        *_TIMESTAMP_COLS,
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
        *_TIMESTAMP_COLS,
    )
    op.create_index("idx_accounts_user", "accounts", ["user_id"])
    op.execute(
        "ALTER TABLE accounts ADD CONSTRAINT uq_accounts_user_label "
        "UNIQUE (user_id, label)"
    )

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
        sa.CheckConstraint("quantity != 0", name="trades_quantity_check"),
    )
    op.create_index("idx_trades_user", "trades", ["user_id"])
    op.create_index("idx_trades_account", "trades", ["account_id"])

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

    # -- Attach updated_at triggers --
    for table in _UPDATED_AT_TABLES:
        op.execute(
            f"""
            CREATE TRIGGER trg_{table}_updated_at
            BEFORE UPDATE ON {table}
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()
            """
        )

    # Drop the old sqlx migration tracking table
    op.execute("DROP TABLE IF EXISTS _sqlx_migrations")


def downgrade() -> None:
    for table in _UPDATED_AT_TABLES:
        op.execute(f"DROP TRIGGER IF EXISTS trg_{table}_updated_at ON {table}")
    op.execute("DROP FUNCTION IF EXISTS set_updated_at()")
    op.drop_table("fx_rates")
    op.drop_table("prices")
    op.drop_table("trades")
    op.drop_table("accounts")
    op.drop_table("instruments")
    op.drop_table("users")
