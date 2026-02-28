// ---------------------------------------------------------------------------
// QR code + terminal output
// ---------------------------------------------------------------------------

/// Print the tunnel URL, QR code, and passcode to stdout.
pub fn print_tunnel_info(project_name: &str, local_port: u16, tunnel_base_url: &str, token: &str) {
    let auth_url = format!("{tunnel_base_url}/?auth={token}");

    println!();
    println!("SDLC UI for '{project_name}'");
    println!("  Local:   http://localhost:{local_port}  (no auth)");
    println!("  Tunnel:  {tunnel_base_url}");
    println!();

    match render_qr(&auth_url) {
        Ok(qr) => print_qr_boxed(&qr),
        Err(_) => {
            // QR rendering failed — fall back to plain URL.
            println!("  {auth_url}");
        }
    }

    println!();
    println!("  Passcode:  {token}");
    println!("  (embedded in QR — scan to access)");
    println!();
    println!("Ctrl+C to stop");
    println!();
}

fn print_qr_boxed(qr: &str) {
    let lines: Vec<&str> = qr.lines().collect();
    let content_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    // 2 spaces padding on each side
    let inner = content_width + 4;
    let border = "─".repeat(inner);

    println!("  ┌{border}┐");
    println!("  │{}│", " ".repeat(inner));
    for line in &lines {
        let pad = inner.saturating_sub(line.chars().count() + 2);
        println!("  │  {line}{}│", " ".repeat(pad));
    }
    println!("  │{}│", " ".repeat(inner));
    println!("  └{border}┘");
}

fn render_qr(url: &str) -> Result<String, qrcode::types::QrError> {
    use qrcode::{render::unicode, QrCode};
    let code = QrCode::new(url.as_bytes())?;
    Ok(code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build())
}
