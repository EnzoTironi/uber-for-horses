import { useState } from "react";
import { Hero } from "../components/Hero";
import { ListingCard } from "../components/ListingCard";
import { useSearchListings } from "../api/hooks";
import type { SearchListingsParams } from "../api/types";
import styles from "./Search.module.css";

const DEFAULT_LAT = 45.5231;
const DEFAULT_LNG = -122.6765;
const DEFAULT_RADIUS = 25;

export function SearchPage() {
  const [latInput, setLatInput] = useState(String(DEFAULT_LAT));
  const [lngInput, setLngInput] = useState(String(DEFAULT_LNG));
  const [radiusInput, setRadiusInput] = useState(String(DEFAULT_RADIUS));
  const [formError, setFormError] = useState<string | null>(null);
  const [params, setParams] = useState<SearchListingsParams>({
    lat: DEFAULT_LAT,
    lng: DEFAULT_LNG,
    radius_km: DEFAULT_RADIUS,
  });

  const { data, isLoading, isError, error, isFetching } =
    useSearchListings(params);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const lat = Number(latInput);
    const lng = Number(lngInput);
    const radius_km = Number(radiusInput);
    if (
      Number.isNaN(lat) ||
      Number.isNaN(lng) ||
      Number.isNaN(radius_km) ||
      radius_km <= 0
    ) {
      setFormError(
        "Please enter valid latitude, longitude, and a positive radius.",
      );
      return;
    }
    setFormError(null);
    setParams({ lat, lng, radius_km });
  }

  function handleLocate() {
    if (!navigator.geolocation) {
      setFormError("Geolocation isn't available in this browser.");
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (position) => {
        const lat = position.coords.latitude;
        const lng = position.coords.longitude;
        setLatInput(lat.toFixed(5));
        setLngInput(lng.toFixed(5));
        setFormError(null);
        setParams({ lat, lng, radius_km: Number(radiusInput) || DEFAULT_RADIUS });
      },
      () => {
        setFormError(
          "Couldn't read your location — enter coordinates manually.",
        );
      },
    );
  }

  return (
    <>
      <Hero>
        <form className={styles.form} onSubmit={handleSubmit}>
          <div className={styles.field}>
            <label htmlFor="lat">Latitude</label>
            <input
              id="lat"
              inputMode="decimal"
              value={latInput}
              onChange={(e) => setLatInput(e.target.value)}
            />
          </div>
          <div className={styles.field}>
            <label htmlFor="lng">Longitude</label>
            <input
              id="lng"
              inputMode="decimal"
              value={lngInput}
              onChange={(e) => setLngInput(e.target.value)}
            />
          </div>
          <div className={styles.field}>
            <label htmlFor="radius">Radius (km)</label>
            <input
              id="radius"
              inputMode="decimal"
              value={radiusInput}
              onChange={(e) => setRadiusInput(e.target.value)}
            />
          </div>
          <button
            type="button"
            className={`btn btn-ghost ${styles["locate-btn"]}`}
            onClick={handleLocate}
          >
            Use my location
          </button>
          <button type="submit" className="btn btn-primary">
            Find horses & carriages
          </button>
          {formError ? (
            <p className={styles["form-error"]}>{formError}</p>
          ) : null}
        </form>
      </Hero>

      <section className={styles.section}>
        <div className={styles["section-heading"]}>
          <h2 className={styles["section-title"]}>Nearby listings</h2>
          <p className={styles["section-meta"]}>
            {params.radius_km} km around ({params.lat.toFixed(2)},{" "}
            {params.lng.toFixed(2)})
            {isFetching ? " · refreshing…" : ""}
          </p>
        </div>

        {isLoading ? (
          <div className={styles["state-panel"]}>
            Saddling up the results…
          </div>
        ) : isError ? (
          <div
            className={`${styles["state-panel"]} ${styles.error ?? ""}`}
          >
            Couldn&apos;t load listings:{" "}
            {error instanceof Error ? error.message : "unknown error"}. Is
            the API running on the configured URL?
          </div>
        ) : data && data.length > 0 ? (
          <div className={styles.grid}>
            {data.map((result) => (
              <ListingCard key={result.listing.id} result={result} />
            ))}
          </div>
        ) : (
          <div className={styles["state-panel"]}>
            No horses or carriages found in this radius yet — try widening
            your search.
          </div>
        )}
      </section>
    </>
  );
}
