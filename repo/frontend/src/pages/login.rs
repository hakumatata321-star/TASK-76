use leptos::*;
use crate::state::auth::AuthState;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState");
    let username = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(Option::<String>::None);
    let loading = create_rw_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        error.set(None);

        let u = username.get();
        let p = password.get();
        let auth = auth.clone();

        spawn_local(async move {
            match crate::api::client::api_post("/auth/login", &serde_json::json!({
                "username": u, "password": p
            })).await {
                Ok((200, json)) => {
                    if let Ok(resp) = serde_wasm_bindgen::from_value::<crate::api::types::LoginResponse>(json) {
                        auth.login(
                            resp.token, resp.csrf_token, resp.user.id,
                            resp.user.username, resp.user.display_name,
                            resp.user.role, resp.user.store_id,
                        );
                        let _ = leptos_router::use_navigate()("/dashboard", Default::default());
                    }
                }
                Ok((_, _)) => error.set(Some("Invalid username or password".to_string())),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    };

    view! {
        <div class="card" style="max-width: 400px; margin: 4rem auto;">
            <h1 style="margin-bottom: 1.5rem;">"FleetReserve Login"</h1>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Username"</label>
                    <input type="text" required autocomplete="username"
                        on:input=move |ev| username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" required autocomplete="current-password"
                        on:input=move |ev| password.set(event_target_value(&ev)) />
                </div>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <button type="submit" class="btn btn-primary" style="width: 100%;"
                    disabled=move || loading.get()>
                    {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                </button>
            </form>
        </div>
    }
}
