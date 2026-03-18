"""add allocations JSONB column to instruments

Revision ID: 0002
Revises: 0001
Create Date: 2026-03-18

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects.postgresql import JSONB


revision: str = "0002"
down_revision: Union[str, None] = "0001"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column(
        "instruments",
        sa.Column("allocations", JSONB, nullable=False, server_default="{}"),
    )


def downgrade() -> None:
    op.drop_column("instruments", "allocations")
