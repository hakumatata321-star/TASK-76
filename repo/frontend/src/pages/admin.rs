use leptos::*;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn AdminPage() -> impl IntoView {
    let users = create_rw_signal(Vec::<serde_json::Value>::new());
    let audit_entries = create_rw_signal(Vec::<serde_json::Value>::new());
    let recovery_code = create_rw_signal(Option::<String>::None);
    let target_user_id = create_rw_signal(String::new());
    let backup_path = create_rw_signal("/data/backups".to_string());
    let restore_path = create_rw_signal(String::new());
    let backup_status = create_rw_signal(Option::<(bool, String)>::None);

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(json) = crate::api::client::api_get("/admin/users").await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                    if let Some(arr) = data.get("users").and_then(|a| a.as_array()) {
                        users.set(arr.clone());
                    }
                }
            }
            if let Ok(json) = crate::api::client::api_get("/audit?limit=20").await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                    if let Some(arr) = data.get("entries").and_then(|a| a.as_array()) {
                        audit_entries.set(arr.clone());
                    }
                }
            }
        });
    });

    let issue_code = move |_| {
        let uid = target_user_id.get();
        spawn_local(async move {
            match crate::api::client::api_post("/admin/recovery-codes", &serde_json::json!({"user_id": uid})).await {
                Ok((200, json)) => {
                    if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                        recovery_code.set(val.get("code").and_then(|c| c.as_str()).map(|s| s.to_string()));
                    }
                }
                _ => {}
            }
        });
    };

    view! {
        <RouteGuard required_role="Administrator">
            <h1>"Administration"</h1>

            <div class="card">
                <h2>"User Management"</h2>
                <table>
                    <thead><tr><th>"ID"</th><th>"Username"</th><th>"Display Name"</th><th>"Role"</th><th>"Store"</th></tr></thead>
                    <tbody>
                        <For
                            each=move || users.get()
                            key=|u| u.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string()
                            children=move |u| view! {
                                <tr>
                                    <td class="masked">{u.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td class="masked">{u.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td>{u.get("display_name").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td>{u.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td>{u.get("store_id").and_then(|v| v.as_str()).unwrap_or("-").to_string()}</td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>

            <div class="card">
                <h2>"Issue Recovery Code"</h2>
                <div style="display: flex; gap: 0.5rem; align-items: end;">
                    <div class="form-group" style="flex: 1; margin-bottom: 0;">
                        <label>"User ID"</label>
                        <input type="text" placeholder="Enter user ID"
                            on:input=move |ev| target_user_id.set(event_target_value(&ev)) />
                    </div>
                    <button class="btn btn-primary" on:click=issue_code>"Issue Code"</button>
                </div>
                <Show when=move || recovery_code.get().is_some()>
                    <div style="margin-top: 1rem; padding: 1rem; background: #f0fdf4; border-radius: 0.5rem;">
                        <strong>"Recovery Code: "</strong>
                        <code style="font-size: 1.25rem;">{move || recovery_code.get().unwrap_or_default()}</code>
                        <p style="font-size: 0.75rem; color: #6b7280;">"Valid for 30 minutes. Provide this code to the user."</p>
                    </div>
                </Show>
            </div>

            <div class="card">
                <h2>"Backup & Restore"</h2>
                <div class="form-group">
                    <label>"Backup destination directory"</label>
                    <input
                        type="text"
                        prop:value=move || backup_path.get()
                        on:input=move |ev| backup_path.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label>"Encrypted backup file path to restore"</label>
                    <input
                        type="text"
                        placeholder="/data/backups/fleetreserve-backup-YYYYMMDDHHMMSS.enc"
                        prop:value=move || restore_path.get()
                        on:input=move |ev| restore_path.set(event_target_value(&ev))
                    />
                </div>
                <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
                    <button class="btn btn-primary"
                        on:click=move |_| {
                            spawn_local(async move {
                                match crate::api::client::api_post("/backup", &serde_json::json!({"path": backup_path.get()})).await {
                                    Ok((200, _)) => backup_status.set(Some((true, "Backup created".into()))),
                                    Ok((_, json)) => {
                                        if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                                            let msg = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Backup failed");
                                            backup_status.set(Some((false, msg.to_string())));
                                        }
                                    }
                                    Err(e) => backup_status.set(Some((false, e))),
                                }
                            });
                        }>"Create Backup"</button>
                    <button class="btn btn-danger"
                        on:click=move |_| {
                            spawn_local(async move {
                                let path = restore_path.get();
                                if path.trim().is_empty() {
                                    backup_status.set(Some((false, "Restore path is required".into())));
                                    return;
                                }
                                match crate::api::client::api_post("/backup/restore", &serde_json::json!({"path": path})).await {
                                    Ok((200, _)) => backup_status.set(Some((true, "Restore completed".into()))),
                                    Ok((_, json)) => {
                                        if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                                            let msg = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Restore failed");
                                            backup_status.set(Some((false, msg.to_string())));
                                        }
                                    }
                                    Err(e) => backup_status.set(Some((false, e))),
                                }
                            });
                        }>"Restore Backup"</button>
                </div>
                <Show when=move || backup_status.get().is_some()>
                    {move || {
                        let (ok, msg) = backup_status.get().unwrap();
                        view! {
                            <p style=format!("margin-top: 0.5rem; color: {};", if ok { "#166534" } else { "#991b1b" })>{msg}</p>
                        }
                    }}
                </Show>
            </div>

            <div class="card">
                <h2>"Audit Log (Recent)"</h2>
                <table>
                    <thead><tr><th>"Time"</th><th>"Actor"</th><th>"Action"</th><th>"Resource"</th></tr></thead>
                    <tbody>
                        <For
                            each=move || audit_entries.get()
                            key=|e| e.get("id").and_then(|i| i.as_i64()).unwrap_or(0).to_string()
                            children=move |e| view! {
                                <tr>
                                    <td style="font-size: 0.75rem;">{e.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td class="masked">{e.get("actor_username").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td>{e.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                    <td>{e.get("resource_type").and_then(|v| v.as_str()).unwrap_or("").to_string()}</td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>
        </RouteGuard>
    }
}
