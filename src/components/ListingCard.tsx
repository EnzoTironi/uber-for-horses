import { Link } from "@tanstack/react-router";
import type { ListingSearchResult } from "../api/types";
import styles from "./ListingCard.module.css";

function formatPrice(cents: number): string {
  return `$${(cents / 100).toFixed(0)}`;
}

export function ListingCard({ result }: { result: ListingSearchResult }) {
  const { listing, distance_km } = result;
  return (
    <Link
      to="/listings/$listingId"
      params={{ listingId: listing.id }}
      className={styles.card}
    >
      <div className={styles["card-photo-wrap"]}>
        <img
          className={styles["card-photo"]}
          src={listing.photo_url}
          alt={listing.name}
          loading="lazy"
        />
        <span className={styles["card-kind"]}>{listing.kind}</span>
        <span className={styles["card-distance"]}>
          {distance_km.toFixed(1)} km away
        </span>
      </div>
      <div className={styles["card-body"]}>
        <h3 className={styles["card-name"]}>{listing.name}</h3>
        <p className={styles["card-description"]}>{listing.description}</p>
        <div className={styles["card-footer"]}>
          <p className={styles["card-price"]}>
            {formatPrice(listing.hourly_price_cents)}{" "}
            <span>/ hour</span>
          </p>
        </div>
      </div>
    </Link>
  );
}
