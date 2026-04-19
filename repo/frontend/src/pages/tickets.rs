use leptos::*;
use leptos_router::*;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn TicketDetailPage() -> impl IntoView {
    let params = use_params_map();
    let ticket = create_rw_signal(Option::<crate::api::types::Ticket>::None);

    create_effect(move |_| {
        let id = params.get().get("id").cloned().unwrap_or_default();
        if !id.is_empty() {
            spawn_local(async move {
                if let Ok(json) = crate::api::client::api_get(&format!("/tickets/{}", id)).await {
                    if let Ok(t) = serde_wasm_bindgen::from_value(json) {
                        ticket.set(Some(t));
                    }
                }
            });
        }
    });

    view! {
        <RouteGuard required_role="Customer">
            <Show when=move || ticket.get().is_some()>
                {move || {
                    let t = ticket.get().unwrap();
                    view! { <crate::components::ticket_display::TicketDisplay ticket=serde_json::to_value(&t).unwrap() /> }
                }}
            </Show>
        </RouteGuard>
    }
}

#[component]
pub fn CheckInPage() -> impl IntoView {
    let ticket_input = create_rw_signal(String::new());
    let status_msg = create_rw_signal(Option::<(bool, String)>::None);
    let redeemed_ticket_id = create_rw_signal(Option::<String>::None);
    let ticket_validity = create_rw_signal(Option::<(String, String)>::None);
    let undo_reason = create_rw_signal(String::new());
    let undo_available = create_rw_signal(false);

    let on_scan_file = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                spawn_local(async move {
                    match crate::api::client::api_upload_file("/tickets/scan", &file, None, None).await {
                        Ok((200, json)) => {
                            if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                                if let Some(v) = val.get("ticket_value").and_then(|s| s.as_str()) {
                                    ticket_input.set(v.to_string());
                                    status_msg.set(Some((true, "QR code decoded — click Redeem to continue.".into())));
                                }
                            }
                        }
                        _ => status_msg.set(Some((false, "Could not decode QR code from image".into()))),
                    }
                });
            }
        }
    };

    let on_redeem = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let input = ticket_input.get();
        status_msg.set(None);

        spawn_local(async move {
            // Try to find ticket by number first, then by ID
            let ticket_id = input.clone();
            if let Ok(json) = crate::api::client::api_get(&format!("/tickets/{}", ticket_id)).await {
                if let Ok(t) = serde_wasm_bindgen::from_value::<crate::api::types::Ticket>(json) {
                    ticket_validity.set(Some((t.valid_from, t.valid_until)));
                }
            }
            match crate::api::client::api_post(&format!("/tickets/{}/redeem", ticket_id), &serde_json::json!({})).await {
                Ok((200, _)) => {
                    status_msg.set(Some((true, "Ticket redeemed successfully!".into())));
                    redeemed_ticket_id.set(Some(ticket_id));
                    undo_available.set(true);
                    // Start 2-minute timer for undo eligibility
                    gloo_timers::callback::Timeout::new(120_000, move || {
                        undo_available.set(false);
                    }).forget();
                }
                Ok((_, json)) => {
                    if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                        let msg = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Redemption failed");
                        status_msg.set(Some((false, msg.to_string())));
                    }
                }
                Err(e) => status_msg.set(Some((false, e))),
            }
        });
    };

    let on_undo = move |_| {
        let reason = undo_reason.get();
        if reason.trim().is_empty() {
            status_msg.set(Some((false, "Undo reason is required".into())));
            return;
        }
        if let Some(tid) = redeemed_ticket_id.get() {
            spawn_local(async move {
                match crate::api::client::api_post(&format!("/tickets/{}/undo", tid), &serde_json::json!({"reason": reason})).await {
                    Ok((200, _)) => {
                        status_msg.set(Some((true, "Redemption undone successfully".into())));
                        undo_available.set(false);
                    }
                    Ok((_, json)) => {
                        if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                            let msg = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Undo failed");
                            status_msg.set(Some((false, msg.to_string())));
                        }
                    }
                    Err(e) => status_msg.set(Some((false, e))),
                }
            });
        }
    };

    view! {
        <RouteGuard required_role="MerchantStaff">
            <h1>"Check-In Station"</h1>
            <div class="card">
                <h2>"Scan or Enter Ticket"</h2>
                <form on:submit=on_redeem>
                    <div class="form-group">
                        <label>"Ticket Number or ID"</label>
                        <input type="text" required placeholder="FR-XXXXXXXX or scan QR code"
                            style="font-size: 1.25rem; padding: 0.75rem;"
                            prop:value=move || ticket_input.get()
                            on:input=move |ev| ticket_input.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <label>"Or scan QR code image (camera or file):"</label>
                        <input type="file" accept="image/*" capture="environment"
                            id="qr-scan-input"
                            on:change=on_scan_file />
                    </div>
                    <button type="submit" class="btn btn-success" style="font-size: 1.1rem; padding: 0.75rem 1.5rem;">"Redeem Ticket"</button>
                </form>
                <Show when=move || ticket_validity.get().is_some()>
                    {move || {
                        let (valid_from, valid_until) = ticket_validity.get().unwrap();
                        view! {
                            <p style="font-size: 0.875rem; color: #4b5563; margin-top: 0.5rem;">
                                "Ticket validity window: "
                                {crate::utils::format::format_datetime(&valid_from)}
                                " - "
                                {crate::utils::format::format_datetime(&valid_until)}
                            </p>
                        }
                    }}
                </Show>

                <Show when=move || status_msg.get().is_some()>
                    {move || {
                        let (success, msg) = status_msg.get().unwrap();
                        view! {
                            <div style=format!("margin-top: 1rem; padding: 1rem; border-radius: 0.5rem; background: {}; color: {};",
                                if success { "#f0fdf4" } else { "#fef2f2" },
                                if success { "#166534" } else { "#991b1b" })>
                                <strong>{if success { "SUCCESS" } else { "ERROR" }}</strong>": " {msg}
                            </div>
                        }
                    }}
                </Show>

                // Supervised undo section
                <Show when=move || undo_available.get()>
                    <div style="margin-top: 1.5rem; padding: 1rem; border: 2px solid #d97706; border-radius: 0.5rem; background: #fffbeb;">
                        <h3 style="color: #92400e;">"Undo Redemption (2-minute window)"</h3>
                        <p style="font-size: 0.875rem; color: #92400e;">"A mandatory reason is required for all undo operations."</p>
                        <div class="form-group">
                            <label>"Reason (required)"</label>
                            <textarea rows="2" required placeholder="Explain why this redemption needs to be undone..."
                                on:input=move |ev| undo_reason.set(event_target_value(&ev))></textarea>
                        </div>
                        <button class="btn btn-danger" on:click=on_undo>"Undo Redemption"</button>
                    </div>
                </Show>
            </div>
        </RouteGuard>
    }
}
