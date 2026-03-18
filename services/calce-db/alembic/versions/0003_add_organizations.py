"""add organizations table and users.organization_id

Revision ID: 0003
Revises: 0002
Create Date: 2026-03-18

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0003"
down_revision: Union[str, None] = "0002"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "organizations",
        sa.Column("id", sa.BigInteger, sa.Identity(always=True), primary_key=True),
        sa.Column("external_id", sa.String(64), unique=True, nullable=False),
        sa.Column("name", sa.String(200)),
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
    )

    op.execute(
        """
        CREATE TRIGGER trg_organizations_updated_at
        BEFORE UPDATE ON organizations
        FOR EACH ROW EXECUTE FUNCTION set_updated_at()
        """
    )

    op.add_column(
        "users",
        sa.Column(
            "organization_id",
            sa.BigInteger,
            sa.ForeignKey("organizations.id"),
            nullable=True,
        ),
    )
    op.create_index("idx_users_organization", "users", ["organization_id"])


def downgrade() -> None:
    op.drop_index("idx_users_organization", "users")
    op.drop_column("users", "organization_id")
    op.execute("DROP TRIGGER IF EXISTS trg_organizations_updated_at ON organizations")
    op.drop_table("organizations")
