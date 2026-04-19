use leptos::*;
use crate::state::auth::AuthState;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState");
    let display_name = auth.display_name;
    let role = auth.role;
    let has_at_least = move |required: &str| -> bool {
        let role_level = |r: &str| -> u8 {
            match r {
                "Customer" => 1,
                "Photographer" => 2,
                "MerchantStaff" => 3,
                "PlatformOps" => 4,
                "Administrator" => 5,
                _ => 0,
            }
        };
        role.get()
            .map(|r| role_level(&r) >= role_level(required))
            .unwrap_or(false)
    };

    view! {
        <RouteGuard required_role="Customer">
            <h1>"Dashboard"</h1>
            <div class="card">
                <p>"Welcome, " {move || display_name.get().unwrap_or("User".into())} "!"</p>
                <p>"Role: " <strong>{move || role.get().unwrap_or_default()}</strong></p>
            </div>

            // Customer view: upcoming reservations and tickets
            <Show when=move || has_at_least("Customer") && role.get().as_deref() != Some("Photographer")>
                <div class="card">
                    <h2>"Your Upcoming Reservations"</h2>
                    <p>"View your reservations and tickets from the navigation menu."</p>
                    <a href="/reservations" class="btn btn-primary">"View Reservations"</a>
                </div>
            </Show>

            // Photographer view: assignments
            <Show when=move || role.get().as_deref() == Some("Photographer")>
                <div class="card">
                    <h2>"Your Assignments"</h2>
                    <a href="/assignments" class="btn btn-primary">"View Assignments"</a>
                </div>
            </Show>

            // Staff view: fleet summary
            <Show when=move || has_at_least("MerchantStaff")>
                <div class="card">
                    <h2>"Store Fleet Management"</h2>
                    <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
                        <a href="/calendar" class="btn btn-primary">"Calendar"</a>
                        <a href="/vehicles" class="btn btn-primary">"Vehicles"</a>
                        <a href="/checkin" class="btn btn-success">"Check-In"</a>
                    </div>
                </div>
            </Show>

            // Ops view: cross-store
            <Show when=move || has_at_least("PlatformOps")>
                <div class="card">
                    <h2>"Platform Operations"</h2>
                    <a href="/exports" class="btn btn-primary">"Exports"</a>
                </div>
            </Show>

            // Admin view
            <Show when=move || has_at_least("Administrator")>
                <div class="card">
                    <h2>"Administration"</h2>
                    <a href="/admin" class="btn btn-primary">"Admin Panel"</a>
                </div>
            </Show>
        </RouteGuard>
    }
}
