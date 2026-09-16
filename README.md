# uber-for-horses

Marketplace connecting horse/carriage owners to riders who book them for
dates, weddings, and events. Rust backend (axum + sqlx/SQLite,
haversine-based nearby search, full booking lifecycle state machine — see
`src/`, `Cargo.toml`) plus the frontend described below. See issue #1 for
the backend spec.

## Paddock & Rein (frontend)

A TanStack Router + TanStack Query single-page app for booking horses and
carriages: search nearby listings, request a time slot, and — as an owner —
manage incoming booking requests.

## Stack

- **Vite + React + TypeScript**
- **@tanstack/react-router** — code-based route tree (`src/router.tsx`)
- **@tanstack/react-query** — all server data fetching/mutations (`src/api/hooks.ts`)
- Plain CSS + CSS Modules for styling (no UI kit) — `src/index.css` holds
  design tokens (palette, type, radii, easing) shared across modules.

## Running locally

```bash
npm install
npm run dev      # http://localhost:5173
npm run build    # type-checks (tsc -b) then builds to dist/
```

Set `VITE_API_BASE_URL` (see `.env.example`) to point at the backend; it
defaults to `http://localhost:8080`.

## API client & auth adapter

All backend calls live in `src/api/api.ts`, one function per endpoint,
typed against `src/api/types.ts`. `src/api/hooks.ts` wraps these in
TanStack Query `useQuery`/`useMutation` hooks consumed by the route
components.

The backend currently authenticates mutating booking actions
(`/bookings/:id/confirm|decline|cancel|complete`) via an `?actor_id=`
query param. Rather than building that query string inline at each call
site, every one of those calls goes through two small functions in
`src/api/auth.ts`:

- `authParams(actorId)` — query-string params for the acting principal
- `authHeaders(actorId)` — headers for the acting principal (a no-op today)

When the backend switches to JWT auth, only `auth.ts` changes — `authParams`
drops the `actor_id` param and `authHeaders` starts returning
`Authorization: Bearer <token>`. No call site in `api.ts` needs to change.

Identity today is mocked with `localStorage` (`src/session/SessionContext.tsx`):
riders are created on first booking, owners "sign in" by creating an owner
record or pasting an existing `owner_id`. This is intentionally simple and
isolated so it can be swapped for real auth later without touching the
route components.

## Routes

- `/` — hero + search form (lat/lng/radius) → `GET /listings/search`,
  results rendered as cards.
- `/listings/$listingId` — listing detail, available time slots
  (`GET /listings/:id/slots`), booking request form (`POST /bookings`).
- `/owner` — owner dashboard: pending "requested" bookings with
  confirm/decline actions, and booking history
  (`GET /owners/:id/bookings`, `POST /bookings/:id/confirm|decline|complete`).

## Design

- **Palette**: warm clay/terracotta, moss green, and gold on a cream/parchment
  base — chosen to read as countryside/equestrian rather than a generic SaaS
  blue.
- **Type**: [Fraunces](https://fonts.google.com/specimen/Fraunces) (display,
  editorial serif with a slight quirk) for headings, paired with
  [Inter](https://fonts.google.com/specimen/Inter) for body/UI text.
- **Hero background**: a photorealistic golden-hour pasture photo from
  Unsplash (`https://images.unsplash.com/photo-1539211070318-e48f041f3e6d`,
  ["Two brown horses eating grasses during golden hour" by Jed Owen](https://unsplash.com/photos/12u0hawVCJ0)),
  with a slow CSS Ken Burns zoom/pan (34s, alternating) plus a second, slower
  drifting radial-gradient light layer — subtle and looping, not a busy
  animated GIF.
