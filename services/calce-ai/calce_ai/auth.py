from __future__ import annotations

import calce
from fastapi import Header, HTTPException, Request


def get_current_user(
    request: Request,
    authorization: str = Header(...),
) -> calce.SecurityContext:
    """FastAPI dependency that validates a Bearer JWT via Rust."""
    if not authorization.startswith("Bearer "):
        raise HTTPException(status_code=401, detail="Missing Bearer token")

    token = authorization.removeprefix("Bearer ")
    auth: calce.AuthService = request.app.state.auth_service

    try:
        return auth.validate_token(token)
    except calce.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid or expired token")
