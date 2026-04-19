use leptos::*;
use leptos_router::*;
use crate::pages::*;
use crate::state::auth::AuthState;
use crate::components::nav::Nav;

#[component]
pub fn App() -> impl IntoView {
    let auth = AuthState::new();
    provide_context(auth.clone());
    let auth_for_effect = auth.clone();
    create_effect(move |_| {
        if auth_for_effect.is_authenticated.get() {
            let auth_clone = auth_for_effect.clone();
            spawn_local(async move {
                if let Ok(json) = crate::api::client::api_get("/auth/me").await {
                    if let Ok(v) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                        if let Some(tok) = v.get("refreshed_token").and_then(|t| t.as_str()) {
                            auth_clone.update_token(tok.to_string());
                        }
                    }
                }
            });
            let auth_interval = auth_for_effect.clone();
            gloo_timers::callback::Interval::new(300_000, move || {
                let auth_tick = auth_interval.clone();
                spawn_local(async move {
                    if let Ok(json) = crate::api::client::api_get("/auth/me").await {
                        if let Ok(v) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                            if let Some(tok) = v.get("refreshed_token").and_then(|t| t.as_str()) {
                                auth_tick.update_token(tok.to_string());
                            }
                        }
                    }
                });
            })
            .forget();
        }
    });

    view! {
        <Router>
            <div class="app-shell">
                <Show when=move || auth.is_authenticated.get()>
                    <Nav />
                </Show>
                <main class="main">
                    <Routes>
                        <Route path="/" view=move || view! { <Redirect path="/dashboard" /> } />
                        <Route path="/login" view=login::LoginPage />
                        <Route path="/dashboard" view=dashboard::DashboardPage />
                        <Route path="/calendar" view=calendar::CalendarPage />
                        <Route path="/reservations" view=reservations::ReservationsPage />
                        <Route path="/vehicles" view=vehicles::VehiclesPage />
                        <Route path="/tickets/:id" view=tickets::TicketDetailPage />
                        <Route path="/checkin" view=tickets::CheckInPage />
                        <Route path="/assignments" view=assignments::AssignmentsPage />
                        <Route path="/admin" view=admin::AdminPage />
                        <Route path="/exports" view=exports::ExportsPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
