export interface Organization {
  id: string;
  name: string | null;
  created_at: string;
}

export interface User {
  id: string;
  email: string | null;
  organization_id: string | null;
  trade_count: number;
}

export interface Instrument {
  id: string;
  name: string | null;
  instrument_type: string;
  currency: string;
  allocations?: {
    sector?: [string, number][];
  };
}

export interface Price {
  date: string;
  price: number;
}

export interface DataStats {
  user_count: number;
  organization_count: number;
  instrument_count: number;
  trade_count: number;
  price_count: number;
  fx_rate_count: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
}

export interface ApiErrorResponse {
  error: string;
  message: string;
}
