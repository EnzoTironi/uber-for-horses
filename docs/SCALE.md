# Scale notes

## Indexes
- `listings(active, lat, lng)`: serves the nearby-search query, which filters on `active` and scans by coordinates.
- `listings(owner_id)`: serves listing-by-owner lookups.
- `bookings(listing_id)`: serves per-listing booking checks (slot availability).
- `bookings(rider_id)`: serves a rider's booking history lookup.

## Connection pool
Max connections set to 10. SQLite allows only one writer at a time; a larger pool mostly helps concurrent reads. 10 is a reasonable default for a single-instance prototype-to-small-production deployment.

## Limitations
SQLite's single-writer constraint means this does not scale past moderate write concurrency (many simultaneous bookings). For real production load with multiple app instances or high write throughput, Postgres is the natural next step — the repository trait boundary already in this codebase makes that swap straightforward without touching service or API code.
