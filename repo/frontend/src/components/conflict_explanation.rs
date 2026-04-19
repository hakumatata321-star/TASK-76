use leptos::*;
use crate::api::types::ConflictResponse;

#[component]
pub fn ConflictExplanation(
    conflict: ConflictResponse,
) -> impl IntoView {
    let reasons = conflict.reasons.clone();
    let alternative_slots = conflict.alternative_slots.clone();
    let alternate_assets = conflict.alternate_assets.clone();
    let slots_for_when = alternative_slots.clone();
    let assets_for_when = alternate_assets.clone();
    view! {
        <div class="conflict-box">
            <h3 style="color: #991b1b; margin-bottom: 0.5rem;">"Reservation Conflict"</h3>
            <ul style="margin: 0.5rem 0; padding-left: 1.5rem;">
                {reasons.iter().map(|r| view! {
                    <li style="margin-bottom: 0.25rem;">{r.message.clone()}</li>
                }).collect_view()}
            </ul>
        </div>

        <Show when=move || !slots_for_when.is_empty()>
            <div class="alternative-box">
                <h3 style="color: #166534; margin-bottom: 0.5rem;">"Nearest Available Time Slots"</h3>
                <ul style="padding-left: 1.5rem;">
                    {alternative_slots.iter().map(|s| view! {
                        <li>
                            {crate::utils::format::format_datetime(&s.start_time)}
                            " to "
                            {crate::utils::format::format_datetime(&s.end_time)}
                        </li>
                    }).collect_view()}
                </ul>
            </div>
        </Show>

        <Show when=move || !assets_for_when.is_empty()>
            <div class="alternative-box">
                <h3 style="color: #166534; margin-bottom: 0.5rem;">"Alternative Vehicles/Bays"</h3>
                <ul style="padding-left: 1.5rem;">
                    {alternate_assets.iter().map(|a| view! {
                        <li>{a.name.clone()} " (" {a.asset_type.clone()} " - " {a.status.clone()} ")"</li>
                    }).collect_view()}
                </ul>
            </div>
        </Show>
    }
}
