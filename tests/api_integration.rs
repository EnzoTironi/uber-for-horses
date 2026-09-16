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

fn get_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn happy_path_full_booking_lifecycle() {
    let app = app();

    // 1. Create owner
    let resp = app
        .clone()
        .oneshot(post_req(
            "/owners",
            json!({ "name": "Alice Owner", "email": "alice@example.com" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let owner = body_json(resp).await;
    let owner_id = owner["id"].as_str().unwrap().to_string();

    // 2. Create listing (San Francisco coordinates)
    let resp = app
        .clone()
        .oneshot(post_req(
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

    // 4. Create rider
    let resp = app
        .clone()
        .oneshot(post_req(
            "/riders",
            json!({ "name": "Bob Rider", "email": "bob@example.com" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let rider = body_json(resp).await;
    let rider_id = rider["id"].as_str().unwrap().to_string();

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

    // 7. Owner confirms booking -> Confirmed, slot becomes booked
    let resp = app
        .clone()
        .oneshot(post_req(
            &format!("/bookings/{booking_id}/confirm?actor_id={owner_id}"),
            json!({}),
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

    // 8. Owner completes booking -> Completed
    let resp = app
        .clone()
        .oneshot(post_req(
            &format!("/bookings/{booking_id}/complete?actor_id={owner_id}"),
            json!({}),
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

    let owner = body_json(
        app.clone()
            .oneshot(post_req(
                "/owners",
                json!({ "name": "Owner1", "email": "owner1@example.com" }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let owner_id = owner["id"].as_str().unwrap().to_string();

    let listing = body_json(
        app.clone()
            .oneshot(post_req(
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

    let rider1 = body_json(
        app.clone()
            .oneshot(post_req(
                "/riders",
                json!({ "name": "Rider1", "email": "rider1@example.com" }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let rider1_id = rider1["id"].as_str().unwrap().to_string();

    let rider2 = body_json(
        app.clone()
            .oneshot(post_req(
                "/riders",
                json!({ "name": "Rider2", "email": "rider2@example.com" }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let rider2_id = rider2["id"].as_str().unwrap().to_string();

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
        .oneshot(post_req(
            &format!("/bookings/{booking1_id}/confirm?actor_id={owner_id}"),
            json!({}),
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

    // Confirming as the wrong owner returns Forbidden (403), not 200.
    let random_actor = Uuid::new_v4();
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
        .oneshot(post_req(
            &format!("/bookings/{booking2_id}/confirm?actor_id={random_actor}"),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cancel_confirmed_booking_frees_the_slot() {
    let app = app();

    let owner = body_json(
        app.clone()
            .oneshot(post_req(
                "/owners",
                json!({ "name": "Owner2", "email": "owner2@example.com" }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let owner_id = owner["id"].as_str().unwrap().to_string();

    let listing = body_json(
        app.clone()
            .oneshot(post_req(
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

    let rider = body_json(
        app.clone()
            .oneshot(post_req(
                "/riders",
                json!({ "name": "Rider3", "email": "rider3@example.com" }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let rider_id = rider["id"].as_str().unwrap().to_string();

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
        .oneshot(post_req(
            &format!("/bookings/{booking_id}/confirm?actor_id={owner_id}"),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Rider cancels the confirmed booking.
    let resp = app
        .clone()
        .oneshot(post_req(
            &format!("/bookings/{booking_id}/cancel?actor_id={rider_id}"),
            json!({}),
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
        .oneshot(post_req(
            &format!("/bookings/{booking_id}/complete?actor_id={owner_id}"),
            json!({}),
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
