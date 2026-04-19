#[path = "../src/api/types.rs"]
mod api_types;

#[test]
fn frontend_api_contract_login_vehicles_and_reservation_shapes() {
    // Contract-level FE<->BE compatibility check without linking backend crate.
    let login_payload = serde_json::json!({
        "token": "token-abc",
        "csrf_token": "csrf-xyz",
        "user": {
            "id": "user-admin-001",
            "username": "admin",
            "display_name": "Administrator",
            "role": "Administrator",
            "store_id": "store-001"
        }
    });
    let login = serde_json::from_value::<api_types::LoginResponse>(login_payload).expect("login dto");
    assert_eq!(login.user.role, "Administrator");
    assert!(!login.token.is_empty());
    assert!(!login.csrf_token.is_empty());

    let vehicles_payload = serde_json::json!([
        {
            "id": "v1",
            "vin": "***1234",
            "license_plate": "***789",
            "make": "Toyota",
            "model": "Van",
            "trim_level": "Base",
            "store_id": "store-001",
            "mileage_miles": 32000,
            "fuel_or_battery_pct": 64.5,
            "status": "available",
            "maintenance_due": null,
            "inspection_due": null,
            "insurance_expiry": "2100-01-01T00:00:00"
        }
    ]);
    let vehicles = serde_json::from_value::<Vec<api_types::MaskedVehicle>>(vehicles_payload)
        .expect("vehicle dto list");
    assert_eq!(vehicles[0].store_id, "store-001");

    let reservation_payload = serde_json::json!({
        "id": "res-001",
        "asset_type": "vehicle",
        "asset_id": "v1",
        "store_id": "store-001",
        "user_id": "user-admin-001",
        "start_time": "2026-10-01T10:00:00",
        "end_time": "2026-10-01T11:00:00",
        "status": "confirmed",
        "ticket_id": null
    });
    let reservation =
        serde_json::from_value::<api_types::Reservation>(reservation_payload).expect("reservation dto");
    assert_eq!(reservation.asset_type, "vehicle");
}
