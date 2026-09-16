// Domain types mirroring the Rust backend's JSON shapes.

export type ListingKind = "horse" | "carriage";

export interface Owner {
  id: string;
  name?: string;
  email?: string;
  [key: string]: unknown;
}

export interface Rider {
  id: string;
  name?: string;
  email?: string;
  [key: string]: unknown;
}

export interface Listing {
  id: string;
  owner_id: string;
  kind: ListingKind;
  name: string;
  description: string;
  photo_url: string;
  hourly_price_cents: number;
  lat: number;
  lng: number;
  active: boolean;
  created_at: string;
}

export interface ListingSearchResult {
  distance_km: number;
  listing: Listing;
}

export interface TimeSlot {
  id: string;
  listing_id: string;
  start_at: string;
  end_at: string;
  [key: string]: unknown;
}

export type BookingStatus =
  | "requested"
  | "confirmed"
  | "declined"
  | "cancelled"
  | "completed";

export interface Booking {
  id: string;
  listing_id: string;
  rider_id: string;
  time_slot_id: string;
  message_from_rider?: string;
  status: BookingStatus;
  created_at?: string;
  [key: string]: unknown;
}

export interface CreateOwnerInput {
  name: string;
  email: string;
}

export interface CreateRiderInput {
  name: string;
  email: string;
}

export interface CreateListingInput {
  owner_id: string;
  kind: ListingKind;
  name: string;
  description: string;
  photo_url: string;
  hourly_price_cents: number;
  lat: number;
  lng: number;
}

export interface CreateSlotInput {
  start_at: string;
  end_at: string;
}

export interface CreateBookingInput {
  listing_id: string;
  rider_id: string;
  time_slot_id: string;
  message_from_rider?: string;
}

export interface SearchListingsParams {
  lat: number;
  lng: number;
  radius_km: number;
}
