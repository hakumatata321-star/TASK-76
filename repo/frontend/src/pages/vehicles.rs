use leptos::*;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn VehiclesPage() -> impl IntoView {
    let vehicles = create_rw_signal(Vec::<crate::api::types::MaskedVehicle>::new());
    let loading = create_rw_signal(true);

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(json) = crate::api::client::api_get("/vehicles").await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                    if let Some(arr) = data.get("vehicles") {
                        if let Ok(list) = serde_json::from_value(arr.clone()) {
                            vehicles.set(list);
                        }
                    }
                }
            }
            loading.set(false);
        });
    });

    view! {
        <RouteGuard required_role="MerchantStaff">
            <h1>"Vehicle Management"</h1>
            <div class="card">
                <h2>"Upload Vehicle Photo"</h2>
                <crate::components::upload_form::UploadForm />
            </div>
            <Show when=move || loading.get()>
                <p>"Loading vehicles..."</p>
            </Show>
            <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem;">
                <For
                    each=move || vehicles.get()
                    key=|v| v.id.clone()
                    children=move |v| view! { <crate::components::vehicle_card::VehicleCard vehicle=v /> }
                />
            </div>
        </RouteGuard>
    }
}
