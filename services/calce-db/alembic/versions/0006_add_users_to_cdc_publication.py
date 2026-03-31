"""add users table to CDC publication

Revision ID: 0006
Revises: 0005
Create Date: 2026-03-31

"""
from typing import Sequence, Union

from alembic import op


revision: str = "0006"
down_revision: Union[str, None] = "0005"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Add users table to the CDC publication (if it exists).
    # The publication may not exist yet — the CDC listener creates it on first
    # connect — so this is a best-effort migration for existing deployments.
    op.execute("""
        DO $$
        BEGIN
            IF EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'calce_cdc_pub') THEN
                ALTER PUBLICATION calce_cdc_pub ADD TABLE users;
            END IF;
        EXCEPTION
            WHEN duplicate_object THEN NULL;
        END $$;
    """)

    # Send all columns on DELETE so the CDC listener can identify the entity.
    op.execute("ALTER TABLE users REPLICA IDENTITY FULL")


def downgrade() -> None:
    op.execute("ALTER TABLE users REPLICA IDENTITY DEFAULT")
    op.execute("""
        DO $$
        BEGIN
            IF EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'calce_cdc_pub') THEN
                ALTER PUBLICATION calce_cdc_pub DROP TABLE users;
            END IF;
        EXCEPTION
            WHEN undefined_object THEN NULL;
        END $$;
    """)
