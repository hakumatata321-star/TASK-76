use leptos::*;

#[component]
pub fn CalendarGrid(
    #[prop(into)] slots: Vec<String>,
    #[prop(into)] view_mode: String,
) -> impl IntoView {
    let cols = if view_mode == "week" { "80px repeat(7, 1fr)" } else { "80px 1fr" };
    view! {
        <div class="calendar-grid" style=format!("grid-template-columns: {};", cols)>
            {slots.iter().map(|s| view! {
                <div class="calendar-slot" style="font-weight: 500; background: #f9fafb; font-size: 0.75rem;">
                    {crate::utils::format::format_time_12h(s)}
                </div>
                <div class="calendar-slot"></div>
            }).collect_view()}
        </div>
    }
}
