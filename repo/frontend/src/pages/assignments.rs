use leptos::*;
use crate::security::route_guard::RouteGuard;

#[component]
pub fn AssignmentsPage() -> impl IntoView {
    let assignments = create_rw_signal(Vec::<crate::api::types::PhotographerAssignment>::new());

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(json) = crate::api::client::api_get("/assignments").await {
                if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                    if let Some(arr) = data.get("assignments") {
                        if let Ok(list) = serde_json::from_value(arr.clone()) {
                            assignments.set(list);
                        }
                    }
                }
            }
        });
    });

    view! {
        <RouteGuard required_role="Photographer">
            <h1>"My Assignments"</h1>
            <div class="card">
                <table>
                    <thead><tr><th>"Job"</th><th>"Store"</th><th>"Start"</th><th>"End"</th><th>"Vehicle"</th><th>"Bay"</th></tr></thead>
                    <tbody>
                        <For
                            each=move || assignments.get()
                            key=|a| a.id.clone()
                            children=move |a| view! {
                                <tr>
                                    <td>{a.job_description.clone()}</td>
                                    <td>{a.store_id.clone()}</td>
                                    <td>{crate::utils::format::format_datetime(&a.start_time)}</td>
                                    <td>{crate::utils::format::format_datetime(&a.end_time)}</td>
                                    <td>{a.vehicle_id.clone().unwrap_or("-".into())}</td>
                                    <td>{a.bay_id.clone().unwrap_or("-".into())}</td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>
        </RouteGuard>
    }
}
