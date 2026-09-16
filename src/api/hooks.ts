import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
  type UseQueryResult,
} from "@tanstack/react-query";
import * as api from "./api";
import type {
  Booking,
  CreateBookingInput,
  Listing,
  ListingSearchResult,
  SearchListingsParams,
  TimeSlot,
} from "./types";

export const queryKeys = {
  listing: (id: string) => ["listing", id] as const,
  listingSlots: (id: string) => ["listing", id, "slots"] as const,
  search: (params: SearchListingsParams) => ["search", params] as const,
  ownerBookings: (ownerId: string) => ["owner", ownerId, "bookings"] as const,
  riderBookings: (riderId: string) => ["rider", riderId, "bookings"] as const,
};

export function useSearchListings(
  params: SearchListingsParams | null,
): UseQueryResult<ListingSearchResult[]> {
  return useQuery({
    queryKey: params ? queryKeys.search(params) : ["search", "idle"],
    queryFn: () => api.searchListings(params as SearchListingsParams),
    enabled: params !== null,
  });
}

export function useListing(id: string | undefined): UseQueryResult<Listing> {
  return useQuery({
    queryKey: queryKeys.listing(id ?? ""),
    queryFn: () => api.getListing(id as string),
    enabled: Boolean(id),
  });
}

export function useListingSlots(
  id: string | undefined,
): UseQueryResult<TimeSlot[]> {
  return useQuery({
    queryKey: queryKeys.listingSlots(id ?? ""),
    queryFn: () => api.getListingSlots(id as string),
    enabled: Boolean(id),
  });
}

export function useCreateBooking(): UseMutationResult<
  Booking,
  Error,
  CreateBookingInput
> {
  return useMutation({
    mutationFn: (input: CreateBookingInput) => api.createBooking(input),
  });
}

export function useOwnerBookings(
  ownerId: string | undefined,
): UseQueryResult<Booking[]> {
  return useQuery({
    queryKey: queryKeys.ownerBookings(ownerId ?? ""),
    queryFn: () => api.getOwnerBookings(ownerId as string),
    enabled: Boolean(ownerId),
  });
}

function useOwnerBookingAction(
  action: (bookingId: string, actorId: string) => Promise<Booking>,
  ownerId: string | undefined,
): UseMutationResult<Booking, Error, string> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (bookingId: string) => action(bookingId, ownerId as string),
    onSuccess: () => {
      if (ownerId) {
        queryClient.invalidateQueries({
          queryKey: queryKeys.ownerBookings(ownerId),
        });
      }
    },
  });
}

export function useConfirmBooking(ownerId: string | undefined) {
  return useOwnerBookingAction(api.confirmBooking, ownerId);
}

export function useDeclineBooking(ownerId: string | undefined) {
  return useOwnerBookingAction(api.declineBooking, ownerId);
}

export function useCompleteBooking(ownerId: string | undefined) {
  return useOwnerBookingAction(api.completeBooking, ownerId);
}
