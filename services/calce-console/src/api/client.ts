import type {
  DataStats,
  Instrument,
  LoginResponse,
  Organization,
  PaginatedResponse,
  Price,
  User,
} from "./types";

const TOKEN_KEY = "access_token";
const REFRESH_KEY = "refresh_token";

let refreshPromise: Promise<void> | null = null;

function buildQuery(params: Record<string, string | number | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) {
      search.set(key, String(value));
    }
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

async function refreshTokens(): Promise<void> {
  const refreshToken = localStorage.getItem(REFRESH_KEY);
  if (!refreshToken) {
    throw new Error("No refresh token");
  }

  const res = await fetch("/auth/refresh", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ refresh_token: refreshToken }),
  });

  if (!res.ok) {
    throw new Error("Refresh failed");
  }

  const data: LoginResponse = await res.json();
  localStorage.setItem(TOKEN_KEY, data.access_token);
  localStorage.setItem(REFRESH_KEY, data.refresh_token);
}

export async function fetchApi<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const token = localStorage.getItem(TOKEN_KEY);
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  let res = await fetch(path, { ...options, headers });

  if (res.status === 401 && token) {
    try {
      if (!refreshPromise) {
        refreshPromise = refreshTokens();
      }
      await refreshPromise;
      refreshPromise = null;

      const newToken = localStorage.getItem(TOKEN_KEY);
      headers["Authorization"] = `Bearer ${newToken}`;
      res = await fetch(path, { ...options, headers });
    } catch {
      refreshPromise = null;
      localStorage.removeItem(TOKEN_KEY);
      localStorage.removeItem(REFRESH_KEY);
      window.location.href = "/login";
      throw new Error("Session expired");
    }
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: "unknown", message: res.statusText }));
    throw new Error(body.message || res.statusText);
  }

  return res.json();
}

export const api = {
  async login(email: string, password: string): Promise<void> {
    const data = await fetchApi<LoginResponse>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });
    localStorage.setItem(TOKEN_KEY, data.access_token);
    localStorage.setItem(REFRESH_KEY, data.refresh_token);
  },

  logout(): void {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_KEY);
  },

  getAccessToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
  },

  isLoggedIn(): boolean {
    return localStorage.getItem(TOKEN_KEY) !== null;
  },

  getStats(): Promise<DataStats> {
    return fetchApi<DataStats>("/v1/data/stats");
  },

  getInstruments(params: {
    offset?: number;
    limit?: number;
    search?: string;
  }): Promise<PaginatedResponse<Instrument>> {
    const qs = buildQuery(params);
    return fetchApi<PaginatedResponse<Instrument>>(`/v1/data/instruments${qs}`);
  },

  getInstrumentPrices(
    instrumentId: string,
    params?: { from?: string; to?: string },
  ): Promise<Price[]> {
    const qs = buildQuery(params ?? {});
    return fetchApi<Price[]>(`/v1/data/instruments/${instrumentId}/prices${qs}`);
  },

  getUsers(params: {
    offset?: number;
    limit?: number;
    search?: string;
  }): Promise<PaginatedResponse<User>> {
    const qs = buildQuery(params);
    return fetchApi<PaginatedResponse<User>>(`/v1/data/users${qs}`);
  },

  getOrganizations(): Promise<Organization[]> {
    return fetchApi<Organization[]>("/v1/organizations");
  },
};
