import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import {
  rpc,
  type UserSummary,
  type CurrentSessionResponse,
  type ChangeEvent,
} from "@/gen/rpc";

export interface AuthState {
  sessionToken: string | null;
  currentUser: UserSummary | null;
  roles: string[];
  permissions: string[];
  isIdleLocked: boolean;
  needsSetup: boolean;
  loading: boolean;
  error: string | null;
}

export interface AuthContextValue extends AuthState {
  login: (username: string, password: string) => Promise<boolean>;
  unlock: (password: string) => Promise<boolean>;
  logout: () => Promise<void>;
  changePassword: (current: string, newPass: string) => Promise<boolean>;
  bootstrapOwner: (
    username: string,
    displayName: string,
    password: string
  ) => Promise<boolean>;
  hasPermission: (permissionCode: string) => boolean;
  isOwner: () => boolean;
  lockManually: () => void;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

const IDLE_TIMEOUT_MS = 15 * 60 * 1000; // 15 minutes
const HEARTBEAT_INTERVAL_MS = 3 * 60 * 1000; // 3 minutes
const SESSION_STORAGE_KEY = "coachify_session_token";

const getStoredToken = (): string | null => {
  if (typeof window !== "undefined" && typeof window.sessionStorage !== "undefined") {
    try {
      return window.sessionStorage.getItem(SESSION_STORAGE_KEY);
    } catch {
      return null;
    }
  }
  return null;
};

const setStoredToken = (token: string | null) => {
  if (typeof window !== "undefined" && typeof window.sessionStorage !== "undefined") {
    try {
      if (token) {
        window.sessionStorage.setItem(SESSION_STORAGE_KEY, token);
      } else {
        window.sessionStorage.removeItem(SESSION_STORAGE_KEY);
      }
    } catch {
      // Ignore storage errors
    }
  }
};

export function AuthProvider({ children }: { children: ReactNode }) {
  const [sessionToken, setSessionToken] = useState<string | null>(getStoredToken);
  const [currentUser, setCurrentUser] = useState<UserSummary | null>(null);
  const [roles, setRoles] = useState<string[]>([]);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [isIdleLocked, setIsIdleLocked] = useState(false);
  const [needsSetup, setNeedsSetup] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const lastActivityRef = useRef<number>(Date.now());
  const tokenRef = useRef<string | null>(sessionToken);
  tokenRef.current = sessionToken;

  const clearError = useCallback(() => setError(null), []);

  // Update session storage whenever token changes
  const updateSessionToken = useCallback((token: string | null) => {
    setSessionToken(token);
    tokenRef.current = token;
    setStoredToken(token);
  }, []);

  // Check setup status
  const checkStatus = useCallback(async () => {
    try {
      const res = await rpc("system.get_status", {});
      if (res.ok && res.data) {
        setNeedsSetup(res.data.needs_setup);
      }
    } catch {
      // Fallback
    }
  }, []);

  // Hydrate session if token exists
  const hydrateSession = useCallback(
    async (token: string) => {
      try {
        const res = await rpc("auth.get_current_session", {}, { sessionToken: token });
        if (res.ok && res.data) {
          const d: CurrentSessionResponse = res.data;
          setCurrentUser(d.user);
          setRoles(d.roles);
          setPermissions(d.permissions);
          setIsIdleLocked(false);
        } else if (res.error?.code === "session_idle_locked") {
          setIsIdleLocked(true);
        } else {
          // Expired or invalid
          updateSessionToken(null);
          setCurrentUser(null);
          setRoles([]);
          setPermissions([]);
        }
      } catch {
        updateSessionToken(null);
      } finally {
        setLoading(false);
      }
    },
    [updateSessionToken]
  );

  useEffect(() => {
    void checkStatus();
    if (sessionToken) {
      void hydrateSession(sessionToken);
    } else {
      setLoading(false);
    }
  }, [checkStatus, hydrateSession, sessionToken]);

  // Login handler
  const login = useCallback(
    async (username: string, password: string): Promise<boolean> => {
      setLoading(true);
      setError(null);
      try {
        const res = await rpc("auth.login", { username, password });
        if (res.ok && res.data) {
          updateSessionToken(res.data.session_token);
          setCurrentUser(res.data.user);
          setRoles(res.data.roles);
          setPermissions(res.data.permissions);
          setIsIdleLocked(false);
          lastActivityRef.current = Date.now();
          setLoading(false);
          return true;
        } else if (res.error) {
          setError(res.error.message || "Login failed");
          setLoading(false);
          return false;
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      }
      setLoading(false);
      return false;
    },
    [updateSessionToken]
  );

  // Unlock handler
  const unlock = useCallback(async (password: string): Promise<boolean> => {
    setError(null);
    const token = tokenRef.current;
    if (!token) return false;

    try {
      const res = await rpc("auth.unlock", { password }, { sessionToken: token });
      if (res.ok) {
        setIsIdleLocked(false);
        lastActivityRef.current = Date.now();
        return true;
      } else if (res.error) {
        setError(res.error.message || "Invalid password");
        return false;
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    }
    return false;
  }, []);

  // Logout handler
  const logout = useCallback(async () => {
    const token = tokenRef.current;
    if (token) {
      try {
        await rpc("auth.logout", {}, { sessionToken: token });
      } catch {
        // Ignore logout errors
      }
    }
    updateSessionToken(null);
    setCurrentUser(null);
    setRoles([]);
    setPermissions([]);
    setIsIdleLocked(false);
  }, [updateSessionToken]);

  // Change password handler
  const changePassword = useCallback(
    async (current: string, newPass: string): Promise<boolean> => {
      setError(null);
      const token = tokenRef.current;
      if (!token) return false;

      try {
        const res = await rpc(
          "auth.change_password",
          { current_password: current, new_password: newPass },
          { sessionToken: token }
        );
        if (res.ok) {
          return true;
        } else if (res.error) {
          setError(res.error.message);
          return false;
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      }
      return false;
    },
    []
  );

  // Bootstrap owner handler
  const bootstrapOwner = useCallback(
    async (username: string, displayName: string, password: string): Promise<boolean> => {
      setLoading(true);
      setError(null);
      try {
        const res = await rpc("system.bootstrap_owner", {
          username,
          display_name: displayName,
          password,
        });
        if (res.ok && res.data) {
          updateSessionToken(res.data.session_token);
          setCurrentUser(res.data.user);
          setRoles(["Owner"]);
          setNeedsSetup(false);
          setIsIdleLocked(false);
          lastActivityRef.current = Date.now();
          setLoading(false);
          return true;
        } else if (res.error) {
          setError(res.error.message);
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      }
      setLoading(false);
      return false;
    },
    [updateSessionToken]
  );

  // Lock manually
  const lockManually = useCallback(() => {
    if (currentUser) {
      setIsIdleLocked(true);
    }
  }, [currentUser]);

  // Permission helpers
  const hasPermission = useCallback(
    (permissionCode: string): boolean => {
      if (roles.includes("Owner")) return true;
      return permissions.includes(permissionCode);
    },
    [roles, permissions]
  );

  const isOwner = useCallback((): boolean => {
    return roles.includes("Owner");
  }, [roles]);

  // Inactivity tracking
  useEffect(() => {
    const handleActivity = () => {
      lastActivityRef.current = Date.now();
    };

    window.addEventListener("mousemove", handleActivity, { passive: true });
    window.addEventListener("keydown", handleActivity, { passive: true });
    window.addEventListener("click", handleActivity, { passive: true });
    window.addEventListener("scroll", handleActivity, { passive: true });

    const idleInterval = setInterval(() => {
      if (tokenRef.current && !isIdleLocked) {
        const inactiveDuration = Date.now() - lastActivityRef.current;
        if (inactiveDuration >= IDLE_TIMEOUT_MS) {
          setIsIdleLocked(true);
        }
      }
    }, 10000);

    return () => {
      window.removeEventListener("mousemove", handleActivity);
      window.removeEventListener("keydown", handleActivity);
      window.removeEventListener("click", handleActivity);
      window.removeEventListener("scroll", handleActivity);
      clearInterval(idleInterval);
    };
  }, [isIdleLocked]);

  // Heartbeat loop every 3 minutes while active
  useEffect(() => {
    const heartbeatInterval = setInterval(() => {
      const token = tokenRef.current;
      if (token && !isIdleLocked) {
        void rpc("auth.heartbeat", {}, { sessionToken: token });
      }
    }, HEARTBEAT_INTERVAL_MS);

    return () => clearInterval(heartbeatInterval);
  }, [isIdleLocked]);

  // Listen to Tauri change events (AC-8)
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<ChangeEvent>("coachify://change", (event) => {
          if (event.payload.topic === "users:changed") {
            // If current user was updated, refresh
            const token = tokenRef.current;
            if (token) {
              void rpc("auth.get_current_session", {}, { sessionToken: token }).then(
                (res) => {
                  if (res.ok && res.data) {
                    setCurrentUser(res.data.user);
                    setRoles(res.data.roles);
                    setPermissions(res.data.permissions);
                  } else if (
                    res.error?.code === "unauthenticated" ||
                    res.error?.code === "session_expired"
                  ) {
                    updateSessionToken(null);
                    setCurrentUser(null);
                    setRoles([]);
                    setPermissions([]);
                  }
                }
              );
            }
          }
        });
      } catch {
        // Running in pure web browser or test environment
      }
    };
    void setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [updateSessionToken]);

  return (
    <AuthContext.Provider
      value={{
        sessionToken,
        currentUser,
        roles,
        permissions,
        isIdleLocked,
        needsSetup,
        loading,
        error,
        login,
        unlock,
        logout,
        changePassword,
        bootstrapOwner,
        hasPermission,
        isOwner,
        lockManually,
        clearError,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

const defaultAuthContext: AuthContextValue = {
  sessionToken: null,
  currentUser: {
    id: "user-default-preview",
    username: "rohan",
    display_name: "Rohan Mehta",
    status: "active",
    must_change_password: false,
    roles: ["Admin"],
    staff_profile_id: null,
    created_at: "2026-09-20T00:00:00Z",
    updated_at: "2026-09-20T00:00:00Z",
  },
  roles: ["Admin"],
  permissions: [],
  isIdleLocked: false,
  needsSetup: false,
  loading: false,
  error: null,
  login: async () => false,
  unlock: async () => false,
  logout: async () => {},
  changePassword: async () => false,
  bootstrapOwner: async () => false,
  hasPermission: () => false,
  isOwner: () => false,
  lockManually: () => {},
  clearError: () => {},
};

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  return context ?? defaultAuthContext;
}
