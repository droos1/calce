import os

DEFAULT_DATABASE_URL = "postgresql://calce:calce@localhost:5433/calce"


def get_database_url() -> str:
    return os.environ.get("DATABASE_URL", DEFAULT_DATABASE_URL)
