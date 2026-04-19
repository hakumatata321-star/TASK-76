#[path = "../src/routes/mod.rs"]
mod routes;

#[test]
fn frontend_router_paths_are_absolute_and_unique() {
    let all = [
        routes::LOGIN,
        routes::DASHBOARD,
        routes::CALENDAR,
        routes::RESERVATIONS,
        routes::VEHICLES,
        routes::CHECKIN,
        routes::ASSIGNMENTS,
        routes::ADMIN,
        routes::EXPORTS,
    ];

    for r in all {
        assert!(r.starts_with('/'), "route must be absolute: {}", r);
    }

    let mut uniq = std::collections::BTreeSet::new();
    for r in all {
        assert!(uniq.insert(r), "duplicate route path: {}", r);
    }
}
