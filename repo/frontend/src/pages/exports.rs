use leptos::*;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn ExportsPage() -> impl IntoView {
    let export_data = create_rw_signal(Option::<serde_json::Value>::None);

    let run_export = move |_| {
        spawn_local(async move {
            // POST because the backend writes an audit entry on export.
            match crate::api::client::api_post("/exports", &serde_json::json!({})).await {
                Ok((200, json)) => {
                    if let Ok(val) = serde_wasm_bindgen::from_value(json) {
                        export_data.set(Some(val));
                    }
                }
                Ok((status, _)) => {
                    leptos::logging::warn!("Export failed with status {}", status);
                }
                Err(e) => {
                    leptos::logging::warn!("Export error: {}", e);
                }
            }
        });
    };

    view! {
        <RouteGuard required_role="PlatformOps">
            <h1>"Data Exports"</h1>
            <div class="card">
                <h2>"Export Data"</h2>
                <p>"Export reservations, vehicles, and bay data across all stores."</p>
                <button class="btn btn-primary" on:click=run_export>"Generate Export"</button>
            </div>
            <Show when=move || export_data.get().is_some()>
                <div class="card">
                    <h2>"Export Result"</h2>
                    <pre style="overflow: auto; max-height: 400px; font-size: 0.75rem;">
                        {move || serde_json::to_string_pretty(&export_data.get().unwrap()).unwrap_or_default()}
                    </pre>
                </div>
            </Show>
        </RouteGuard>
    }
}
