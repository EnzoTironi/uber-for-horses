import { authHeaders, authParams } from "./auth";
import type {
  Booking,
  CreateBookingInput,
  CreateListingInput,
  CreateOwnerInput,
  CreateRiderInput,
  CreateSlotInput,
  Listing,
  ListingSearchResult,
  Owner,
  Rider,
  SearchListingsParams,
  TimeSlot,
} from "./types";

export const API_BASE_URL: string =
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ??
  "http://localhost:8080";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function request<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init.headers ?? {}),
    },
  });

  if (!res.ok) {
    let message = `Request failed with status ${res.status}`;
    try {
      const body = await res.text();
      if (body) message = body;
    } catch {
      // ignore
    }
    throw new ApiError(res.status, message);
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return (await res.json()) as T;
}

function qs(params: Record<string, string | number | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) search.set(key, String(value));
  }
  const str = search.toString();
  return str ? `?${str}` : "";
}

// ---- Owners ----

export function createOwner(input: CreateOwnerInput): Promise<Owner> {
  return request<Owner>("/owners", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function getOwner(id: string): Promise<Owner> {
  return request<Owner>(`/owners/${id}`);
}

export function getOwnerBookings(ownerId: string): Promise<Booking[]> {
  return request<Booking[]>(`/owners/${ownerId}/bookings`);
}

// ---- Riders ----

export function createRider(input: CreateRiderInput): Promise<Rider> {
  return request<Rider>("/riders", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function getRider(id: string): Promise<Rider> {
  return request<Rider>(`/riders/${id}`);
}

export function getRiderBookings(riderId: string): Promise<Booking[]> {
  return request<Booking[]>(`/riders/${riderId}/bookings`);
}

// ---- Listings ----

export function createListing(input: CreateListingInput): Promise<Listing> {
  return request<Listing>("/listings", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function getListing(id: string): Promise<Listing> {
  return request<Listing>(`/listings/${id}`);
}

export function searchListings(
  params: SearchListingsParams,
): Promise<ListingSearchResult[]> {
  const query = qs({
    lat: params.lat,
    lng: params.lng,
    radius_km: params.radius_km,
  });
  return request<ListingSearchResult[]>(`/listings/search${query}`);
}

// ---- Slots ----

export function createSlot(
  listingId: string,
  input: CreateSlotInput,
): Promise<TimeSlot> {
  return request<TimeSlot>(`/listings/${listingId}/slots`, {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function getListingSlots(listingId: string): Promise<TimeSlot[]> {
  return request<TimeSlot[]>(`/listings/${listingId}/slots`);
}

// ---- Bookings ----

export function createBooking(input: CreateBookingInput): Promise<Booking> {
  return request<Booking>("/bookings", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

/**
 * Shared shape for the owner-actioned booking transitions below. `actorId`
 * is the owner's id today; it flows through `authParams`/`authHeaders` so
 * that swapping to JWT auth later only touches those two functions.
 */
function actOnBooking(bookingId: string, action: string, actorId: string) {
  const query = qs(authParams(actorId));
  return request<Booking>(`/bookings/${bookingId}/${action}${query}`, {
    method: "POST",
    headers: authHeaders(actorId),
  });
}

export function confirmBooking(
  bookingId: string,
  actorId: string,
): Promise<Booking> {
  return actOnBooking(bookingId, "confirm", actorId);
}

export function declineBooking(
  bookingId: string,
  actorId: string,
): Promise<Booking> {
  return actOnBooking(bookingId, "decline", actorId);
}

export function cancelBooking(
  bookingId: string,
  actorId: string,
): Promise<Booking> {
  return actOnBooking(bookingId, "cancel", actorId);
}

export function completeBooking(
  bookingId: string,
  actorId: string,
): Promise<Booking> {
  return actOnBooking(bookingId, "complete", actorId);
}
