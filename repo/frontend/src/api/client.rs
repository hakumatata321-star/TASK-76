use crate::state::auth::AuthState;
use leptos::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FormData, Headers, Request, RequestInit, Response};

pub async fn api_get(path: &str) -> Result<JsValue, String> {
    let auth = use_context::<AuthState>().ok_or("No auth context")?;
    let opts = RequestInit::new();
    opts.set_method("GET");

    let headers = Headers::new().map_err(|e| format!("{:?}", e))?;
    if let Some(token) = auth.token.get_untracked() {
        headers.set("Authorization", &format!("Bearer {}", token)).map_err(|e| format!("{:?}", e))?;
    }
    opts.set_headers(&headers);

    let url = format!("/api{}", path);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| format!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Not a response")?;

    if resp.status() == 401 {
        auth.logout();
        return Err("Session expired".to_string());
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?).await.map_err(|e| format!("{:?}", e))?;
    Ok(json)
}

pub async fn api_post(path: &str, body: &serde_json::Value) -> Result<(u16, JsValue), String> {
    let auth = use_context::<AuthState>().ok_or("No auth context")?;
    let opts = RequestInit::new();
    opts.set_method("POST");

    let headers = Headers::new().map_err(|e| format!("{:?}", e))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("{:?}", e))?;
    if let Some(token) = auth.token.get_untracked() {
        headers.set("Authorization", &format!("Bearer {}", token)).map_err(|e| format!("{:?}", e))?;
    }
    if let Some(csrf) = auth.csrf_token.get_untracked() {
        headers.set("X-CSRF-Token", &csrf).map_err(|e| format!("{:?}", e))?;
    }
    opts.set_headers(&headers);

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    opts.set_body(&JsValue::from_str(&body_str));

    let url = format!("/api{}", path);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| format!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Not a response")?;

    let status = resp.status();
    if status == 401 {
        auth.logout();
        return Err("Session expired".to_string());
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?).await.map_err(|e| format!("{:?}", e))?;
    Ok((status, json))
}

pub async fn api_put(path: &str, body: &serde_json::Value) -> Result<(u16, JsValue), String> {
    let auth = use_context::<AuthState>().ok_or("No auth context")?;
    let opts = RequestInit::new();
    opts.set_method("PUT");

    let headers = Headers::new().map_err(|e| format!("{:?}", e))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("{:?}", e))?;
    if let Some(token) = auth.token.get_untracked() {
        headers.set("Authorization", &format!("Bearer {}", token)).map_err(|e| format!("{:?}", e))?;
    }
    if let Some(csrf) = auth.csrf_token.get_untracked() {
        headers.set("X-CSRF-Token", &csrf).map_err(|e| format!("{:?}", e))?;
    }
    opts.set_headers(&headers);

    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    opts.set_body(&JsValue::from_str(&body_str));

    let url = format!("/api{}", path);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| format!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Not a response")?;
    let status = resp.status();
    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?).await.map_err(|e| format!("{:?}", e))?;
    Ok((status, json))
}

pub async fn api_upload_file(
    path: &str,
    file: &File,
    vehicle_id: Option<&str>,
    store_id: Option<&str>,
) -> Result<(u16, JsValue), String> {
    let auth = use_context::<AuthState>().ok_or("No auth context")?;
    let opts = RequestInit::new();
    opts.set_method("POST");

    let headers = Headers::new().map_err(|e| format!("{:?}", e))?;
    if let Some(token) = auth.token.get_untracked() {
        headers.set("Authorization", &format!("Bearer {}", token)).map_err(|e| format!("{:?}", e))?;
    }
    if let Some(csrf) = auth.csrf_token.get_untracked() {
        headers.set("X-CSRF-Token", &csrf).map_err(|e| format!("{:?}", e))?;
    }
    opts.set_headers(&headers);

    let form = FormData::new().map_err(|e| format!("{:?}", e))?;
    form.append_with_blob_and_filename("file", file, &file.name()).map_err(|e| format!("{:?}", e))?;
    if let Some(v) = vehicle_id {
        form.append_with_str("vehicle_id", v).map_err(|e| format!("{:?}", e))?;
    }
    if let Some(s) = store_id {
        form.append_with_str("store_id", s).map_err(|e| format!("{:?}", e))?;
    }
    opts.set_body(&form.into());

    let url = format!("/api{}", path);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{:?}", e))?;
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| format!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "Not a response")?;

    let status = resp.status();
    if status == 401 {
        auth.logout();
        return Err("Session expired".to_string());
    }
    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?).await.map_err(|e| format!("{:?}", e))?;
    Ok((status, json))
}
