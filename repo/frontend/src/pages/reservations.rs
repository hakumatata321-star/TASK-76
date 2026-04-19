use leptos::*;
use crate::security::route_guard::RouteGuard;
use crate::state::auth::AuthState;
use crate::components::conflict_explanation::ConflictExplanation;

#[component]
pub fn ReservationsPage() -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState");
    let asset_type = create_rw_signal("vehicle".to_string());
    let asset_id = create_rw_signal(String::new());
    let start_time = create_rw_signal(String::new());
    let end_time = create_rw_signal(String::new());
    let store_id = create_rw_signal(auth.store_id.get_untracked().unwrap_or("store-001".into()));
    let result = create_rw_signal(Option::<Result<serde_json::Value, crate::api::types::ConflictResponse>>::None);
    let loading = create_rw_signal(false);
    let reservations = create_rw_signal(Vec::<crate::api::types::Reservation>::new());

    // Load existing reservations
    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(json) = crate::api::client::api_get("/reservations").await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                    if let Some(arr) = data.get("reservations") {
                        if let Ok(list) = serde_json::from_value(arr.clone()) {
                            reservations.set(list);
                        }
                    }
                }
            }
        });
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        result.set(None);

        let req = serde_json::json!({
            "asset_type": asset_type.get(),
            "asset_id": asset_id.get(),
            "store_id": store_id.get(),
            "start_time": start_time.get(),
            "end_time": end_time.get(),
        });

        spawn_local(async move {
            match crate::api::client::api_post("/reservations", &req).await {
                Ok((201, json)) => {
                    if let Ok(val) = serde_wasm_bindgen::from_value(json) {
                        result.set(Some(Ok(val)));
                    }
                }
                Ok((409, json)) => {
                    if let Ok(conflict) = serde_wasm_bindgen::from_value(json) {
                        result.set(Some(Err(conflict)));
                    }
                }
                Ok((_, _)) => result.set(Some(Ok(serde_json::json!({"error": "Unexpected response"})))),
                Err(e) => result.set(Some(Ok(serde_json::json!({"error": e})))),
            }
            loading.set(false);
        });
    };

    view! {
        <RouteGuard required_role="Customer">
            <h1>"Reservations"</h1>

            <div class="card">
                <h2>"Create Reservation"</h2>
                <form on:submit=on_submit>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                        <div class="form-group">
                            <label>"Asset Type"</label>
                            <select on:change=move |ev| asset_type.set(event_target_value(&ev))>
                                <option value="vehicle">"Vehicle"</option>
                                <option value="bay">"Service Bay"</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <label>"Asset ID"</label>
                            <input type="text" required placeholder="Enter asset ID"
                                on:input=move |ev| asset_id.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-group">
                            <label>"Start Time"</label>
                            <input type="datetime-local" required step="900"
                                on:input=move |ev| start_time.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-group">
                            <label>"End Time"</label>
                            <input type="datetime-local" required step="900"
                                on:input=move |ev| end_time.set(event_target_value(&ev)) />
                        </div>
                    </div>
                    <p style="font-size: 0.75rem; color: #6b7280; margin-bottom: 0.5rem;">"Times must be in 15-minute increments within business hours (7:00 AM - 7:00 PM)"</p>
                    <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                        {move || if loading.get() { "Creating..." } else { "Create Reservation" }}
                    </button>
                </form>
            </div>

            // Show result
            <Show when=move || result.get().is_some()>
                {move || match result.get().unwrap() {
                    Ok(val) => {
                        if let Some(ticket) = val.get("ticket") {
                            view! {
                                <div class="card" style="border-color: #16a34a;">
                                    <h2 style="color: #16a34a;">"Reservation Confirmed!"</h2>
                                    <crate::components::ticket_display::TicketDisplay ticket=ticket.clone() />
                                </div>
                            }.into_view()
                        } else {
                            view! { <div class="card"><p>"Reservation created"</p></div> }.into_view()
                        }
                    }
                    Err(conflict) => {
                        view! { <ConflictExplanation conflict=conflict /> }.into_view()
                    }
                }}
            </Show>

            // Existing reservations list
            <div class="card">
                <h2>"Your Reservations"</h2>
                <table>
                    <thead><tr>
                        <th>"Asset"</th><th>"Start"</th><th>"End"</th><th>"Status"</th><th>"Ticket"</th>
                    </tr></thead>
                    <tbody>
                        <For
                            each=move || reservations.get()
                            key=|r| r.id.clone()
                            children=move |r| view! {
                                <tr>
                                    <td>{r.asset_type.clone()} " / " {r.asset_id.clone()}</td>
                                    <td>{crate::utils::format::format_datetime(&r.start_time)}</td>
                                    <td>{crate::utils::format::format_datetime(&r.end_time)}</td>
                                    <td><crate::components::status_badge::StatusBadge status=r.status.clone() /></td>
                                    <td>
                                        {r.ticket_id.as_ref().map(|tid| view! {
                                            <a href=format!("/tickets/{}", tid) class="btn btn-primary" style="font-size: 0.75rem; padding: 0.25rem 0.5rem;">"View Ticket"</a>
                                        })}
                                    </td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>
        </RouteGuard>
    }
}
