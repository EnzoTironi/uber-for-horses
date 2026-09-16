// Thin auth adapter.
//
// Today the backend identifies the acting principal (owner or rider) via an
// `actor_id` query param on mutating booking endpoints. A parallel effort may
// later swap this for a JWT bearer header. Every call in `api.ts` goes
// through `authParams()` / `authHeaders()` below instead of building
// `actor_id=` by hand, so migrating to JWT auth means changing these two
// functions (and how `getCurrentActorId` is populated) — not every call site.

const ACTOR_ID_STORAGE_KEY = "uber-for-horses:actor-id";

export function getCurrentActorId(): string | null {
  try {
    return window.localStorage.getItem(ACTOR_ID_STORAGE_KEY);
  } catch {
    return null;
  }
}

export function setCurrentActorId(actorId: string | null): void {
  try {
    if (actorId) {
      window.localStorage.setItem(ACTOR_ID_STORAGE_KEY, actorId);
    } else {
      window.localStorage.removeItem(ACTOR_ID_STORAGE_KEY);
    }
  } catch {
    // localStorage unavailable (e.g. SSR/private mode) — no-op.
  }
}

/**
 * Query-string params identifying the acting principal for a mutating
 * request. Under today's backend this is `?actor_id=...`. When the backend
 * switches to JWT auth, this becomes `{}` (no query param) and the token
 * moves into `authHeaders()` below.
 */
export function authParams(actorId: string): Record<string, string> {
  return { actor_id: actorId };
}

/**
 * Headers to attach to every request for the acting principal. A no-op
 * today; once the backend moves to JWT auth this returns
 * `{ Authorization: `Bearer ${token}` }`.
 */
export function authHeaders(_actorId?: string): Record<string, string> {
  return {};
}
