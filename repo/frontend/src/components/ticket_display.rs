use leptos::*;

/// Generates a standards-compliant QR SVG data URI.
fn generate_qr_svg(data: &str) -> String {
    let qr = match qrcodegen::QrCode::encode_text(data, qrcodegen::QrCodeEcc::Medium) {
        Ok(q) => q,
        Err(_) => return String::new(),
    };
    let border = 2;
    let module_px = 8;
    let size = qr.size();
    let total = (size + border * 2) * module_px;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {0} {0}" width="200" height="200"><rect width="{0}" height="{0}" fill="white"/>"#,
        total
    );
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                let px = (x + border) * module_px;
                let py = (y + border) * module_px;
                svg.push_str(&format!(
                    r#"<rect x="{px}" y="{py}" width="{m}" height="{m}" fill="black"/>"#,
                    px = px,
                    py = py,
                    m = module_px
                ));
            }
        }
    }
    svg.push_str("</svg>");
    let encoded = simple_b64(svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", encoded)
}

fn simple_b64(input: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let a = input[i] as u32;
        let b = if i+1 < input.len() { input[i+1] as u32 } else { 0 };
        let c = if i+2 < input.len() { input[i+2] as u32 } else { 0 };
        let n = (a << 16) | (b << 8) | c;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if i+1 < input.len() { T[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if i+2 < input.len() { T[(n & 63) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

#[component]
pub fn TicketDisplay(
    ticket: serde_json::Value,
) -> impl IntoView {
    let number = ticket.get("ticket_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let valid_from = ticket.get("valid_from").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let valid_until = ticket.get("valid_until").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let redeemed = ticket.get("redeemed").and_then(|v| v.as_bool()).unwrap_or(false);
    let qr_data = ticket.get("qr_data").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let qr_src = generate_qr_svg(&qr_data);

    view! {
        <div class="ticket-display">
            <img src=qr_src alt="QR Code for ticket" style="width: 200px; height: 200px; margin: 1rem auto; display: block; image-rendering: pixelated;" />
            <div class="ticket-number">{number}</div>
            <div class="validity-window">
                "Valid: " {crate::utils::format::format_datetime(&valid_from)}
                " - " {crate::utils::format::format_datetime(&valid_until)}
            </div>
            <div style="margin-top: 0.5rem;">
                {if redeemed {
                    view! { <span class="badge" style="background: #fef3c7; color: #92400e;">"REDEEMED"</span> }.into_view()
                } else {
                    view! { <span class="badge badge-available">"VALID"</span> }.into_view()
                }}
            </div>
        </div>
    }
}
