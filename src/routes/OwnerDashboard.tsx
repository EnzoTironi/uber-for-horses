import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import * as api from "../api/api";
import {
  useCompleteBooking,
  useConfirmBooking,
  useDeclineBooking,
  useOwnerBookings,
} from "../api/hooks";
import { useSession } from "../session/SessionContext";
import type { Booking, BookingStatus } from "../api/types";
import styles from "./OwnerDashboard.module.css";

const STATUS_LABEL: Record<BookingStatus, string> = {
  requested: "Requested",
  confirmed: "Confirmed",
  declined: "Declined",
  cancelled: "Cancelled",
  completed: "Completed",
};

function StatusBadge({ status }: { status: BookingStatus }) {
  return <span className={`badge badge-${status}`}>{STATUS_LABEL[status]}</span>;
}

function IdentityPanel() {
  const { owner, setOwner } = useSession();
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [existingId, setExistingId] = useState("");

  const createOwner = useMutation({
    mutationFn: () => api.createOwner({ name, email }),
    onSuccess: (created) => setOwner({ id: created.id, name, email }),
  });

  const lookupOwner = useMutation({
    mutationFn: (id: string) => api.getOwner(id),
    onSuccess: (found, id) =>
      setOwner({
        id,
        name: (found.name as string | undefined) ?? "Owner",
        email: (found.email as string | undefined) ?? "",
      }),
  });

  if (owner) {
    return (
      <div className={styles["identity-panel"]}>
        <div className={styles["identity-active"]}>
          <div>
            <p className={styles["identity-name"]}>{owner.name}</p>
            <p className={styles["identity-email"]}>{owner.email}</p>
            <p className={styles["booking-id"]}>owner_id: {owner.id}</p>
          </div>
          <button
            type="button"
            className="btn btn-ghost"
            onClick={() => setOwner(null)}
          >
            Switch owner
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles["identity-panel"]}>
      <h2>Sign in as an owner</h2>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          createOwner.mutate();
        }}
      >
        <div className={styles["form-group"]}>
          <label htmlFor="owner-name">Name</label>
          <input
            id="owner-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
        </div>
        <div className={styles["form-group"]}>
          <label htmlFor="owner-email">Email</label>
          <input
            id="owner-email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
          />
        </div>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={createOwner.isPending}
        >
          {createOwner.isPending ? "Creating…" : "Create owner & continue"}
        </button>
      </form>

      <div style={{ marginTop: "1.25rem" }}>
        <p className={styles["identity-email"]} style={{ marginBottom: "0.5rem" }}>
          Already have an owner id?
        </p>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (existingId.trim()) lookupOwner.mutate(existingId.trim());
          }}
          style={{ display: "flex", gap: "0.6rem" }}
        >
          <input
            value={existingId}
            onChange={(e) => setExistingId(e.target.value)}
            placeholder="owner_id"
            style={{
              flex: 1,
              border: "1.5px solid var(--color-line)",
              borderRadius: "var(--radius-sm)",
              padding: "0.55rem 0.75rem",
            }}
          />
          <button type="submit" className="btn btn-secondary btn-small">
            Use id
          </button>
        </form>
      </div>

      {createOwner.isError || lookupOwner.isError ? (
        <p style={{ color: "var(--color-status-declined)", marginTop: "0.75rem", fontSize: "0.85rem" }}>
          Couldn&apos;t reach the API. Is the backend running?
        </p>
      ) : null}
    </div>
  );
}

function BookingActions({ booking, ownerId }: { booking: Booking; ownerId: string }) {
  const confirmMutation = useConfirmBooking(ownerId);
  const declineMutation = useDeclineBooking(ownerId);
  const completeMutation = useCompleteBooking(ownerId);

  if (booking.status === "requested") {
    return (
      <div className={styles["booking-actions"]}>
        <button
          type="button"
          className="btn btn-primary btn-small"
          disabled={confirmMutation.isPending}
          onClick={() => confirmMutation.mutate(booking.id)}
        >
          Confirm
        </button>
        <button
          type="button"
          className="btn btn-decline btn-small"
          disabled={declineMutation.isPending}
          onClick={() => declineMutation.mutate(booking.id)}
        >
          Decline
        </button>
      </div>
    );
  }

  if (booking.status === "confirmed") {
    return (
      <div className={styles["booking-actions"]}>
        <button
          type="button"
          className="btn btn-secondary btn-small"
          disabled={completeMutation.isPending}
          onClick={() => completeMutation.mutate(booking.id)}
        >
          Mark completed
        </button>
      </div>
    );
  }

  return null;
}

export function OwnerDashboardPage() {
  const { owner } = useSession();
  const bookingsQuery = useOwnerBookings(owner?.id);
  const bookings = bookingsQuery.data ?? [];

  const requested = bookings.filter((b) => b.status === "requested");
  const history = bookings.filter((b) => b.status !== "requested");

  return (
    <div className={styles.wrap}>
      <div className={styles.header}>
        <h1 className={styles.title}>Owner dashboard</h1>
        <p className={styles.subtitle}>
          Review booking requests for your horses and carriages, and confirm
          or decline them.
        </p>
      </div>

      <IdentityPanel />

      {owner ? (
        <>
          <h2 className={styles["section-title"]}>
            Pending requests
            {requested.length > 0 ? (
              <span className={styles["count-chip"]}>{requested.length}</span>
            ) : null}
          </h2>

          {bookingsQuery.isLoading ? (
            <p className={styles["empty-state"]}>Loading bookings…</p>
          ) : bookingsQuery.isError ? (
            <p className={styles["empty-state"]}>
              Couldn&apos;t load bookings. Is the API reachable?
            </p>
          ) : requested.length === 0 ? (
            <div className={styles["empty-state"]}>
              No pending requests right now.
            </div>
          ) : (
            <div className={styles["booking-list"]}>
              {requested.map((booking) => (
                <div
                  key={booking.id}
                  className={`${styles["booking-card"]} ${styles.requested}`}
                >
                  <div className={styles["booking-info"]}>
                    <StatusBadge status={booking.status} />
                    <p className={styles["booking-id"]}>
                      booking_id: {booking.id}
                    </p>
                    {booking.message_from_rider ? (
                      <p className={styles["booking-message"]}>
                        “{booking.message_from_rider}”
                      </p>
                    ) : null}
                  </div>
                  <BookingActions booking={booking} ownerId={owner.id} />
                </div>
              ))}
            </div>
          )}

          <h2 className={styles["section-title"]}>History</h2>
          {history.length === 0 ? (
            <div className={styles["empty-state"]}>
              No past bookings yet.
            </div>
          ) : (
            <div className={styles["history-list"]}>
              {history.map((booking) => (
                <div key={booking.id} className={styles["history-row"]}>
                  <span className={styles["booking-id"]}>{booking.id}</span>
                  <StatusBadge status={booking.status} />
                </div>
              ))}
            </div>
          )}
        </>
      ) : null}
    </div>
  );
}
