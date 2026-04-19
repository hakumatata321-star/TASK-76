use leptos::*;
use crate::state::auth::AuthState;

#[component]
pub fn RouteGuard(
    #[prop(into)] required_role: String,
    children: ChildrenFn,
) -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState not provided");
    let role_for_when = required_role.clone();
    let auth_for_when = auth.clone();
    let auth_for_fallback = auth.clone();

    view! {
        <Show
            when=move || auth_for_when.is_authenticated.get() && auth_for_when.has_role(&role_for_when)
            fallback=move || view! {
                <Show
                    when=move || !auth_for_fallback.is_authenticated.get()
                    fallback=|| view! { <div class="card"><h2>"Access Denied"</h2><p>"You do not have permission to view this page."</p></div> }
                >
                    <div class="card"><h2>"Please log in"</h2><p>"Authentication required to access this page."</p><a href="/login">"Go to Login"</a></div>
                </Show>
            }
        >
            {children()}
        </Show>
    }
}
