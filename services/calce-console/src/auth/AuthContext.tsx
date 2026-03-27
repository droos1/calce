import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { Navigate, Outlet, useNavigate } from "react-router";
import { api } from "../api/client";

interface AuthUser {
  userId: string;
  email?: string;
  role: string;
}

interface AuthContextValue {
  isAuthenticated: boolean;
  user: AuthUser | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function decodeJwtPayload(token: string): { sub: string; role: string; email?: string; exp: number } {
  const base64 = token.split(".")[1];
  const json = atob(base64);
  return JSON.parse(json);
}

function getUserFromToken(): AuthUser | null {
  const token = api.getAccessToken();
  if (!token) return null;

  try {
    const payload = decodeJwtPayload(token);
    if (payload.exp * 1000 < Date.now()) {
      api.logout();
      return null;
    }
    return { userId: payload.sub, email: payload.email, role: payload.role };
  } catch {
    api.logout();
    return null;
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(() => getUserFromToken());
  const navigate = useNavigate();

  const login = useCallback(async (email: string, password: string) => {
    await api.login(email, password);
    setUser(getUserFromToken());
  }, []);

  const logout = useCallback(() => {
    api.logout();
    setUser(null);
    navigate("/login");
  }, [navigate]);

  const value = useMemo<AuthContextValue>(
    () => ({
      isAuthenticated: user !== null,
      user,
      login,
      logout,
    }),
    [user, login, logout],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return ctx;
}

export function ProtectedRoute() {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}

export default AuthProvider;
