use leptos::*;
use crate::state::auth::AuthState;

#[component]
pub fn UploadForm(
    #[prop(optional)] vehicle_id: Option<String>,
) -> impl IntoView {
    let auth = use_context::<AuthState>().expect("AuthState");
    let status = create_rw_signal(Option::<(bool, String)>::None);
    let file_name = create_rw_signal(String::new());
    let selected_file = create_rw_signal(Option::<web_sys::File>::None);

    let on_file_change = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let name = file.name();
                let size = file.size() as usize;
                if size > 10 * 1024 * 1024 {
                    status.set(Some((false, "File exceeds 10 MB limit".into())));
                    return;
                }
                let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
                if ext != "jpg" && ext != "jpeg" && ext != "png" {
                    status.set(Some((false, "Only JPEG and PNG files are accepted".into())));
                    return;
                }
                file_name.set(name);
                selected_file.set(Some(file));
                status.set(Some((true, "File selected, ready to upload".into())));
            }
        }
    };

    let on_upload = move |_| {
        let file = selected_file.get();
        if file.is_none() {
            status.set(Some((false, "Select a file first".into())));
            return;
        }
        let f = file.unwrap();
        let v = vehicle_id.clone();
        let store = auth.store_id.get_untracked();
        spawn_local(async move {
            match crate::api::client::api_upload_file(
                "/uploads",
                &f,
                v.as_deref(),
                store.as_deref(),
            )
            .await
            {
                Ok((201, _)) => status.set(Some((true, "Upload completed".into()))),
                Ok((_, json)) => {
                    if let Ok(val) = serde_wasm_bindgen::from_value::<serde_json::Value>(json) {
                        let msg = val.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Upload failed");
                        status.set(Some((false, msg.to_string())));
                    }
                }
                Err(e) => status.set(Some((false, e))),
            }
        });
    };

    view! {
        <div class="form-group">
            <label>"Upload Photo (JPEG/PNG, max 10 MB)"</label>
            <input type="file" accept="image/jpeg,image/png" on:change=on_file_change />
            <button type="button" class="btn btn-primary" style="margin-top: 0.5rem;" on:click=on_upload>
                "Upload"
            </button>
        </div>
        <Show when=move || status.get().is_some()>
            {move || {
                let (ok, msg) = status.get().unwrap();
                view! {
                    <p style=format!("font-size: 0.875rem; color: {};", if ok { "#166534" } else { "#991b1b" })>
                        {msg}
                    </p>
                }
            }}
        </Show>
    }
}
