import { useState } from "react";
import { Link, useParams } from "@tanstack/react-router";
import { useMutation } from "@tanstack/react-query";
import * as api from "../api/api";
import {
  useCreateBooking,
  useListing,
  useListingSlots,
} from "../api/hooks";
import { useSession } from "../session/SessionContext";
import type { TimeSlot } from "../api/types";
import styles from "./ListingDetail.module.css";

function formatPrice(cents: number): string {
  return `$${(cents / 100).toFixed(0)}`;
}

function formatSlot(slot: TimeSlot): { range: string; date: string } {
  const start = new Date(slot.start_at);
  const end = new Date(slot.end_at);
  const dateFmt = new Intl.DateTimeFormat(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
  const timeFmt = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
  return {
    date: dateFmt.format(start),
    range: `${timeFmt.format(start)} – ${timeFmt.format(end)}`,
  };
}

export function ListingDetailPage() {
  const { listingId } = useParams({ from: "/listings/$listingId" });
  const listingQuery = useListing(listingId);
  const slotsQuery = useListingSlots(listingId);
  const { rider, setRider } = useSession();

  const [selectedSlotId, setSelectedSlotId] = useState<string | null>(null);
  const [name, setName] = useState(rider?.name ?? "");
  const [email, setEmail] = useState(rider?.email ?? "");
  const [message, setMessage] = useState("");

  const ensureRider = useMutation({
    mutationFn: async () => {
      if (rider) return rider;
      const created = await api.createRider({ name, email });
      const stored = { id: created.id, name, email };
      setRider(stored);
      return stored;
    },
  });

  const createBooking = useCreateBooking();

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedSlotId || !listingId) return;
    const activeRider = await ensureRider.mutateAsync();
    createBooking.mutate({
      listing_id: listingId,
      rider_id: activeRider.id,
      time_slot_id: selectedSlotId,
      message_from_rider: message || undefined,
    });
  }

  const listing = listingQuery.data;
  const slots = slotsQuery.data ?? [];
  const canSubmit =
    Boolean(selectedSlotId) && name.trim() !== "" && email.trim() !== "";

  return (
    <div className={styles.wrap}>
      <div>
        <Link to="/" className={styles["back-link"]}>
          ← Back to search
        </Link>

        {listingQuery.isLoading ? (
          <p>Loading listing…</p>
        ) : listingQuery.isError ? (
          <p>
            Couldn&apos;t load this listing. It may not exist, or the API
            isn&apos;t reachable.
          </p>
        ) : listing ? (
          <>
            <img
              className={styles.photo}
              src={listing.photo_url}
              alt={listing.name}
            />
            <span className={styles["kind-tag"]}>{listing.kind}</span>
            <h1 className={styles.title}>{listing.name}</h1>
            <p className={styles.description}>{listing.description}</p>
            <p className={styles.price}>
              {formatPrice(listing.hourly_price_cents)} <span>/ hour</span>
            </p>
          </>
        ) : null}
      </div>

      <div className={styles.panel}>
        <h2 className={styles["panel-title"]}>Request a booking</h2>

        {slotsQuery.isLoading ? (
          <p className={styles["empty-slots"]}>Loading available times…</p>
        ) : slots.length === 0 ? (
          <p className={styles["empty-slots"]}>
            No open time slots right now — check back soon.
          </p>
        ) : (
          <div className={styles.slots}>
            {slots.map((slot) => {
              const { range, date } = formatSlot(slot);
              const selected = slot.id === selectedSlotId;
              return (
                <button
                  key={slot.id}
                  type="button"
                  className={`${styles.slot} ${selected ? styles.selected : ""}`}
                  onClick={() => setSelectedSlotId(slot.id)}
                >
                  <span className={styles["slot-range"]}>{range}</span>
                  <span className={styles["slot-date"]}>{date}</span>
                </button>
              );
            })}
          </div>
        )}

        <form onSubmit={handleSubmit}>
          <div className={styles["form-group"]}>
            <label htmlFor="rider-name">Your name</label>
            <input
              id="rider-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={Boolean(rider)}
              required
            />
          </div>
          <div className={styles["form-group"]}>
            <label htmlFor="rider-email">Your email</label>
            <input
              id="rider-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={Boolean(rider)}
              required
            />
          </div>
          <div className={styles["form-group"]}>
            <label htmlFor="rider-message">Message to the owner (optional)</label>
            <textarea
              id="rider-message"
              rows={3}
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Tell them about your group, occasion, or any questions…"
            />
          </div>
          <button
            type="submit"
            className="btn btn-primary"
            disabled={
              !canSubmit ||
              createBooking.isPending ||
              ensureRider.isPending
            }
            style={{ width: "100%" }}
          >
            {createBooking.isPending || ensureRider.isPending
              ? "Sending request…"
              : "Request booking"}
          </button>
        </form>

        {createBooking.isSuccess ? (
          <p className={styles["result-panel"]}>
            Request sent! Status:{" "}
            <strong>{createBooking.data.status}</strong>. The owner will
            confirm or decline shortly.
          </p>
        ) : null}
        {createBooking.isError || ensureRider.isError ? (
          <p className={`${styles["result-panel"]} ${styles.error ?? ""}`}>
            Something went wrong submitting your request. Please try again.
          </p>
        ) : null}
      </div>
    </div>
  );
}
