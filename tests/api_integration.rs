use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use uber_for_horses::build_in_memory_state;

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn app() -> axum::Router {
    uber_for_horses::api::router(build_in_memory_state())
}

fn post_req(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn post_req_auth(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn post_req_auth_header(uri: &str, body: Value, header_value: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", header_value)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn get_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

/// Sign up a new owner (via /auth/signup) and return (owner_id, token).
async fn signup_owner(
    app: &axum::Router,
    name: &str,
    email: &str,
    password: &str,
) -> (String, String) {
    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/signup",
            json!({ "name": name, "email": email, "password": password, "role": "owner" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();

    // Look the owner up via /owners? We don't have a "whoami" endpoint, so create
    // the owner and independently fetch its id by decoding via login flow is
    // unnecessary here — instead we still need the id for legacy assertions
    // (e.g. matching against listing.owner_id). We get it by also creating the
    // owner record's id from token claims isn't exposed, so we create via the
    // plain /owners endpoint isn't ideal either. Simplest: search owners is not
    // available, so we instead decode the JWT payload ourselves (base64) to pull
    // the `sub` claim - this keeps tests decoupled from any extra endpoint.
    let claims_b64 = token.split('.').nth(1).unwrap();
    let claims_json = base64_url_decode(claims_b64);
    let claims: Value = serde_json::from_slice(&claims_json).unwrap();
    let owner_id = claims["sub"].as_str().unwrap().to_string();

    (owner_id, token)
}

/// Sign up a new rider (via /auth/signup) and return (rider_id, token).
async fn signup_rider(
    app: &axum::Router,
    name: &str,
    email: &str,
    password: &str,
) -> (String, String) {
    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/signup",
            json!({ "name": name, "email": email, "password": password, "role": "rider" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();

    let claims_b64 = token.split('.').nth(1).unwrap();
    let claims_json = base64_url_decode(claims_b64);
    let claims: Value = serde_json::from_slice(&claims_json).unwrap();
    let rider_id = claims["sub"].as_str().unwrap().to_string();

    (rider_id, token)
}

/// Minimal base64url decoder (no padding) sufficient for decoding a JWT payload
/// segment in tests, without adding a base64 crate dependency.
fn base64_url_decode(input: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let mut bits: u32 = 0;
    let mut bit_count = 0u32;
    let mut out = Vec::new();
    for &b in input.as_bytes() {
        let v = table[b as usize];
        if v == 255 {
            continue;
        }
        bits = (bits << 6) | v as u32;
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            out.push(((bits >> bit_count) & 0xFF) as u8);
        }
    }
    out
}

#[tokio::test]
async fn happy_path_full_booking_lifecycle() {
    let app = app();

    // 1. Sign up owner
    let (owner_id, owner_token) =
        signup_owner(&app, "Alice Owner", "alice@example.com", "hunter2").await;

    // 2. Create listing (San Francisco coordinates)
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            "/listings",
            json!({
                "owner_id": owner_id,
                "kind": "horse",
                "name": "Majestic Steed",
                "description": "A very good horse",
                "photo_url": "https://example.com/horse.png",
                "hourly_price_cents": 5000,
                "lat": 37.7749,
                "lng": -122.4194
            }),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let listing = body_json(resp).await;
    let listing_id = listing["id"].as_str().unwrap().to_string();
    assert_eq!(listing["active"], json!(true));

    // 3. Add a time slot (2 hours)
    let start_at = Utc::now() + Duration::days(1);
    let end_at = start_at + Duration::hours(2);
    let resp = app
        .clone()
        .oneshot(post_req(
            &format!("/listings/{listing_id}/slots"),
            json!({ "start_at": start_at, "end_at": end_at }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let slot = body_json(resp).await;
    let slot_id = slot["id"].as_str().unwrap().to_string();
    assert_eq!(slot["is_booked"], json!(false));

    // 4. Sign up rider
    let (rider_id, _rider_token) =
        signup_rider(&app, "Bob Rider", "bob@example.com", "swordfish").await;

    // 5. search_nearby finds the listing (Oakland is ~13km from SF)
    let resp = app
        .clone()
        .oneshot(get_req(
            "/listings/search?lat=37.8044&lng=-122.2712&radius_km=25",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let results = body_json(resp).await;
    let results_arr = results.as_array().unwrap();
    assert_eq!(results_arr.len(), 1);
    assert_eq!(results_arr[0]["listing"]["id"], json!(listing_id));
    assert!(results_arr[0]["distance_km"].as_f64().unwrap() > 0.0);

    // A too-small radius should find nothing.
    let resp = app
        .clone()
        .oneshot(get_req(
            "/listings/search?lat=37.8044&lng=-122.2712&radius_km=1",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let results = body_json(resp).await;
    assert_eq!(results.as_array().unwrap().len(), 0);

    // 6. Rider creates booking -> status Requested
    let resp = app
        .clone()
        .oneshot(post_req(
            "/bookings",
            json!({
                "listing_id": listing_id,
                "rider_id": rider_id,
                "time_slot_id": slot_id,
                "message_from_rider": "Excited for the ride!"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let booking = body_json(resp).await;
    let booking_id = booking["id"].as_str().unwrap().to_string();
    assert_eq!(booking["status"], json!("requested"));
    // 2 hours * 5000 cents/hr = 10000 cents
    assert_eq!(booking["total_price_cents"], json!(10000));

    // Slot should still be unbooked at this point.
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/listings/{listing_id}/slots")))
        .await
        .unwrap();
    let slots = body_json(resp).await;
    assert_eq!(slots[0]["is_booked"], json!(false));

    // 7. Owner confirms booking (via JWT identity) -> Confirmed, slot becomes booked
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking_id}/confirm"),
            json!({}),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let booking = body_json(resp).await;
    assert_eq!(booking["status"], json!("confirmed"));

    let resp = app
        .clone()
        .oneshot(get_req(&format!("/listings/{listing_id}/slots")))
        .await
        .unwrap();
    let slots = body_json(resp).await;
    assert_eq!(slots[0]["is_booked"], json!(true));

    // 8. Owner completes booking (via JWT identity) -> Completed
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking_id}/complete"),
            json!({}),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let booking = body_json(resp).await;
    assert_eq!(booking["status"], json!("completed"));

    // Sanity check: GET /bookings/:id reflects the same state.
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/bookings/{booking_id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let booking = body_json(resp).await;
    assert_eq!(booking["status"], json!("completed"));

    // Rider's booking history includes this booking.
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/riders/{rider_id}/bookings")))
        .await
        .unwrap();
    let rider_bookings = body_json(resp).await;
    assert_eq!(rider_bookings.as_array().unwrap().len(), 1);

    // Owner's booking history includes this booking.
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/owners/{owner_id}/bookings")))
        .await
        .unwrap();
    let owner_bookings = body_json(resp).await;
    assert_eq!(owner_bookings.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn booking_an_already_booked_slot_returns_conflict() {
    let app = app();

    let (owner_id, owner_token) =
        signup_owner(&app, "Owner1", "owner1@example.com", "pw-owner1").await;

    let listing = body_json(
        app.clone()
            .oneshot(post_req_auth(
                "/listings",
                json!({
                    "owner_id": owner_id,
                    "kind": "carriage",
                    "name": "Fancy Carriage",
                    "description": "desc",
                    "photo_url": "https://example.com/c.png",
                    "hourly_price_cents": 8000,
                    "lat": 40.0,
                    "lng": -73.0
                }),
                &owner_token,
            ))
            .await
            .unwrap(),
    )
    .await;
    let listing_id = listing["id"].as_str().unwrap().to_string();

    let start_at = Utc::now() + Duration::days(2);
    let end_at = start_at + Duration::hours(1);
    let slot = body_json(
        app.clone()
            .oneshot(post_req(
                &format!("/listings/{listing_id}/slots"),
                json!({ "start_at": start_at, "end_at": end_at }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let slot_id = slot["id"].as_str().unwrap().to_string();

    let (rider1_id, _rider1_token) =
        signup_rider(&app, "Rider1", "rider1@example.com", "pw-rider1").await;
    let (rider2_id, _rider2_token) =
        signup_rider(&app, "Rider2", "rider2@example.com", "pw-rider2").await;

    // Rider1 books and owner confirms -> slot becomes booked.
    let booking1 = body_json(
        app.clone()
            .oneshot(post_req(
                "/bookings",
                json!({
                    "listing_id": listing_id,
                    "rider_id": rider1_id,
                    "time_slot_id": slot_id,
                    "message_from_rider": null
                }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let booking1_id = booking1["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking1_id}/confirm"),
            json!({}),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Rider2 tries to book the now-confirmed (booked) slot -> Conflict.
    let resp = app
        .clone()
        .oneshot(post_req(
            "/bookings",
            json!({
                "listing_id": listing_id,
                "rider_id": rider2_id,
                "time_slot_id": slot_id,
                "message_from_rider": null
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    // Confirming as a different (non-owning) owner returns Forbidden (403), not 200.
    let (_random_owner_id, random_owner_token) =
        signup_owner(&app, "RandomOwner", "random-owner@example.com", "pw-random").await;
    // Create a second booking request on a fresh slot to test wrong-owner confirm.
    let start_at2 = Utc::now() + Duration::days(3);
    let end_at2 = start_at2 + Duration::hours(1);
    let slot2 = body_json(
        app.clone()
            .oneshot(post_req(
                &format!("/listings/{listing_id}/slots"),
                json!({ "start_at": start_at2, "end_at": end_at2 }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let slot2_id = slot2["id"].as_str().unwrap().to_string();

    let booking2 = body_json(
        app.clone()
            .oneshot(post_req(
                "/bookings",
                json!({
                    "listing_id": listing_id,
                    "rider_id": rider2_id,
                    "time_slot_id": slot2_id,
                    "message_from_rider": null
                }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let booking2_id = booking2["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking2_id}/confirm"),
            json!({}),
            &random_owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cancel_confirmed_booking_frees_the_slot() {
    let app = app();

    let (owner_id, owner_token) =
        signup_owner(&app, "Owner2", "owner2@example.com", "pw-owner2").await;

    let listing = body_json(
        app.clone()
            .oneshot(post_req_auth(
                "/listings",
                json!({
                    "owner_id": owner_id,
                    "kind": "horse",
                    "name": "Trusty Horse",
                    "description": "desc",
                    "photo_url": "https://example.com/h.png",
                    "hourly_price_cents": 3000,
                    "lat": 10.0,
                    "lng": 10.0
                }),
                &owner_token,
            ))
            .await
            .unwrap(),
    )
    .await;
    let listing_id = listing["id"].as_str().unwrap().to_string();

    let start_at = Utc::now() + Duration::days(1);
    let end_at = start_at + Duration::hours(3);
    let slot = body_json(
        app.clone()
            .oneshot(post_req(
                &format!("/listings/{listing_id}/slots"),
                json!({ "start_at": start_at, "end_at": end_at }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let slot_id = slot["id"].as_str().unwrap().to_string();

    let (rider_id, rider_token) =
        signup_rider(&app, "Rider3", "rider3@example.com", "pw-rider3").await;

    let booking = body_json(
        app.clone()
            .oneshot(post_req(
                "/bookings",
                json!({
                    "listing_id": listing_id,
                    "rider_id": rider_id,
                    "time_slot_id": slot_id,
                    "message_from_rider": null
                }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let booking_id = booking["id"].as_str().unwrap().to_string();

    // Confirm it.
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking_id}/confirm"),
            json!({}),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Rider cancels the confirmed booking.
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking_id}/cancel"),
            json!({}),
            &rider_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let booking = body_json(resp).await;
    assert_eq!(booking["status"], json!("cancelled"));

    // Slot should be freed again.
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/listings/{listing_id}/slots")))
        .await
        .unwrap();
    let slots = body_json(resp).await;
    assert_eq!(slots[0]["is_booked"], json!(false));

    // Completing an already-cancelled booking should fail with Conflict (409).
    let resp = app
        .clone()
        .oneshot(post_req_auth(
            &format!("/bookings/{booking_id}/complete"),
            json!({}),
            &owner_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn getting_missing_resources_returns_404() {
    let app = app();

    let missing_id = Uuid::new_v4();
    let resp = app
        .clone()
        .oneshot(get_req(&format!("/owners/{missing_id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp = app
        .clone()
        .oneshot(get_req(&format!("/bookings/{missing_id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------- New auth tests ----------

#[tokio::test]
async fn signup_then_login_succeeds() {
    let app = app();

    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/signup",
            json!({ "name": "Carol Owner", "email": "carol@example.com", "password": "s3cret!", "role": "owner" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let signup_body = body_json(resp).await;
    assert!(signup_body["token"].as_str().unwrap().len() > 0);

    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/login",
            json!({ "email": "carol@example.com", "password": "s3cret!", "role": "owner" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let login_body = body_json(resp).await;
    assert!(login_body["token"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let app = app();

    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/signup",
            json!({ "name": "Dave Owner", "email": "dave@example.com", "password": "correct-horse", "role": "owner" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(post_req(
            "/auth/login",
            json!({ "email": "dave@example.com", "password": "wrong-password", "role": "owner" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_endpoint_without_token_returns_401() {
    let app = app();

    let missing_id = Uuid::new_v4();
    let resp = app
        .clone()
        .oneshot(post_req(
            &format!("/bookings/{missing_id}/confirm"),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_endpoint_with_garbage_token_returns_401() {
    let app = app();

    let missing_id = Uuid::new_v4();
    let resp = app
        .clone()
        .oneshot(post_req_auth_header(
            &format!("/bookings/{missing_id}/confirm"),
            json!({}),
            "Bearer this-is-not-a-valid-jwt",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rider_cannot_create_listing_returns_403() {
    let app = app();

    let (owner_id, _owner_token) =
        signup_owner(&app, "Eve Owner", "eve@example.com", "pw-eve").await;
    let (_rider_id, rider_token) =
        signup_rider(&app, "Frank Rider", "frank@example.com", "pw-frank").await;

    let resp = app
        .clone()
        .oneshot(post_req_auth(
            "/listings",
            json!({
                "owner_id": owner_id,
                "kind": "horse",
                "name": "Sneaky Listing",
                "description": "desc",
                "photo_url": "https://example.com/x.png",
                "hourly_price_cents": 1000,
                "lat": 0.0,
                "lng": 0.0
            }),
            &rider_token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
