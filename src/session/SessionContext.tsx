import { createContext, useCallback, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";

const RIDER_STORAGE_KEY = "uber-for-horses:rider";
const OWNER_STORAGE_KEY = "uber-for-horses:owner";

export interface StoredIdentity {
  id: string;
  name: string;
  email: string;
}

interface SessionContextValue {
  rider: StoredIdentity | null;
  owner: StoredIdentity | null;
  setRider: (rider: StoredIdentity | null) => void;
  setOwner: (owner: StoredIdentity | null) => void;
}

const SessionContext = createContext<SessionContextValue | null>(null);

function readStored(key: string): StoredIdentity | null {
  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as StoredIdentity) : null;
  } catch {
    return null;
  }
}

function writeStored(key: string, value: StoredIdentity | null): void {
  try {
    if (value) {
      window.localStorage.setItem(key, JSON.stringify(value));
    } else {
      window.localStorage.removeItem(key);
    }
  } catch {
    // ignore
  }
}

export function SessionProvider({ children }: { children: ReactNode }) {
  const [rider, setRiderState] = useState<StoredIdentity | null>(() =>
    readStored(RIDER_STORAGE_KEY),
  );
  const [owner, setOwnerState] = useState<StoredIdentity | null>(() =>
    readStored(OWNER_STORAGE_KEY),
  );

  const setRider = useCallback((value: StoredIdentity | null) => {
    writeStored(RIDER_STORAGE_KEY, value);
    setRiderState(value);
  }, []);

  const setOwner = useCallback((value: StoredIdentity | null) => {
    writeStored(OWNER_STORAGE_KEY, value);
    setOwnerState(value);
  }, []);

  const value = useMemo(
    () => ({ rider, owner, setRider, setOwner }),
    [rider, owner, setRider, setOwner],
  );

  return (
    <SessionContext.Provider value={value}>{children}</SessionContext.Provider>
  );
}

export function useSession(): SessionContextValue {
  const ctx = useContext(SessionContext);
  if (!ctx) {
    throw new Error("useSession must be used within a SessionProvider");
  }
  return ctx;
}
