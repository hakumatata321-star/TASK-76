#[test]
fn frontend_app_and_pages_have_direct_test_references() {
    let app = include_str!("../src/app.rs");
    assert!(app.contains("pub fn App"));
    assert!(app.contains("Route path=\"/login\""));
    assert!(app.contains("Route path=\"/admin\""));

    let pages = [
        ("login", include_str!("../src/pages/login.rs"), "pub fn LoginPage"),
        ("dashboard", include_str!("../src/pages/dashboard.rs"), "pub fn DashboardPage"),
        ("calendar", include_str!("../src/pages/calendar.rs"), "pub fn CalendarPage"),
        ("reservations", include_str!("../src/pages/reservations.rs"), "pub fn ReservationsPage"),
        ("vehicles", include_str!("../src/pages/vehicles.rs"), "pub fn VehiclesPage"),
        ("tickets", include_str!("../src/pages/tickets.rs"), "pub fn TicketDetailPage"),
        ("assignments", include_str!("../src/pages/assignments.rs"), "pub fn AssignmentsPage"),
        ("admin", include_str!("../src/pages/admin.rs"), "pub fn AdminPage"),
        ("exports", include_str!("../src/pages/exports.rs"), "pub fn ExportsPage"),
    ];

    for (name, src, signature) in pages {
        assert!(src.contains(signature), "missing page signature for {}", name);
        if name != "login" {
            assert!(src.contains("RouteGuard"), "page {} should be route-guarded", name);
        }
    }
}

#[test]
fn frontend_components_and_guards_have_direct_test_references() {
    let components = [
        ("nav", include_str!("../src/components/nav.rs"), "pub fn Nav"),
        (
            "calendar_grid",
            include_str!("../src/components/calendar_grid.rs"),
            "pub fn CalendarGrid",
        ),
        (
            "conflict_explanation",
            include_str!("../src/components/conflict_explanation.rs"),
            "pub fn ConflictExplanation",
        ),
        (
            "ticket_display",
            include_str!("../src/components/ticket_display.rs"),
            "pub fn TicketDisplay",
        ),
        (
            "vehicle_card",
            include_str!("../src/components/vehicle_card.rs"),
            "pub fn VehicleCard",
        ),
        (
            "upload_form",
            include_str!("../src/components/upload_form.rs"),
            "pub fn UploadForm",
        ),
        (
            "status_badge",
            include_str!("../src/components/status_badge.rs"),
            "pub fn StatusBadge",
        ),
    ];
    for (name, src, signature) in components {
        assert!(src.contains(signature), "missing component signature for {}", name);
    }

    let route_guard = include_str!("../src/security/route_guard.rs");
    assert!(route_guard.contains("pub fn RouteGuard"));
    assert!(route_guard.contains("Access Denied"));

    let api_client = include_str!("../src/api/client.rs");
    assert!(api_client.contains("pub async fn api_get"));
    assert!(api_client.contains("pub async fn api_post"));
    assert!(api_client.contains("pub async fn api_put"));
    assert!(api_client.contains("pub async fn api_upload_file"));
}
