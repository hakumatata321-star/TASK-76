use leptos::*;

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let class = match status.as_str() {
        "available" | "confirmed" | "valid" => "badge badge-available",
        "reserved" => "badge badge-reserved",
        "on-rent" => "badge badge-on-rent",
        "in-repair" => "badge badge-in-repair",
        "decommissioned" | "cancelled" => "badge badge-decommissioned",
        _ => "badge",
    };
    view! { <span class=class>{status}</span> }
}
