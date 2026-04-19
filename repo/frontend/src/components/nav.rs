use leptos::*;
use crate::state::auth::AuthState;

#[component]
pub fn Nav() -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState");
    let auth_for_logout = auth.clone();

    let on_logout = move |_| {
        spawn_local(async move {
            let _ = crate::api::client::api_post("/auth/logout", &serde_json::json!({})).await;
        });
        auth_for_logout.logout();
        let _ = leptos_router::use_navigate()("/login", Default::default());
    };

    let auth_for_staff = auth.clone();
    let auth_for_photo = auth.clone();
    let auth_for_ops = auth.clone();
    let auth_for_admin = auth.clone();
    let auth_for_name = auth.clone();
    let auth_for_role = auth.clone();

    view! {
        <nav class="nav">
            <div style="display: flex; align-items: center; gap: 1rem;">
                <strong>"FleetReserve"</strong>
                <a href="/dashboard">"Dashboard"</a>
                <a href="/calendar">"Calendar"</a>
                <a href="/reservations">"Reservations"</a>
                <Show when=move || auth_for_staff.has_role("MerchantStaff")>
                    <a href="/vehicles">"Vehicles"</a>
                    <a href="/checkin">"Check-In"</a>
                </Show>
                <Show when=move || auth_for_photo.role.get().as_deref() == Some("Photographer")>
                    <a href="/assignments">"Assignments"</a>
                </Show>
                <Show when=move || auth_for_ops.has_role("PlatformOps")>
                    <a href="/exports">"Exports"</a>
                </Show>
                <Show when=move || auth_for_admin.has_role("Administrator")>
                    <a href="/admin">"Admin"</a>
                </Show>
            </div>
            <div style="display: flex; align-items: center; gap: 0.75rem;">
                <span style="font-size: 0.875rem;">{move || auth_for_name.display_name.get().unwrap_or_default()}</span>
                <span class="badge" style="background: rgba(255,255,255,0.2);">{move || auth_for_role.role.get().unwrap_or_default()}</span>
                <button class="btn" style="background: rgba(255,255,255,0.15); color: white; font-size: 0.75rem;" on:click=on_logout>"Logout"</button>
            </div>
        </nav>
    }
}
