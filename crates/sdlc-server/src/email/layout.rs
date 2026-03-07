use super::theme;

/// Wrap `body_html` in the full Ponder email shell.
///
/// Uses table-based layout with all CSS inlined for maximum email client
/// compatibility. No `<style>` blocks — Gmail strips them.
pub fn wrap(preheader: &str, body_html: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="dark">
<meta name="supported-color-schemes" content="dark">
<title>Ponder</title>
</head>
<body style="margin:0;padding:0;background-color:{bg};font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;">
<!-- preheader (hidden preview text) -->
<div style="display:none;max-height:0;overflow:hidden;mso-hide:all;">{preheader}</div>
<!-- outer wrapper -->
<table width="100%" cellpadding="0" cellspacing="0" border="0" bgcolor="{bg}" style="background-color:{bg};">
  <tr>
    <td align="center" style="padding:40px 16px;">
      <!-- 600px centered container -->
      <table width="600" cellpadding="0" cellspacing="0" border="0" style="max-width:600px;width:100%;">
        <!-- wordmark header -->
        <tr>
          <td align="center" style="padding:0 0 24px 0;">
            <span style="font-size:28px;font-weight:700;letter-spacing:4px;color:{primary};font-family:monospace;">PONDER</span>
          </td>
        </tr>
        <!-- content card -->
        <tr>
          <td bgcolor="{card}" style="background-color:{card};border:1px solid {card_border};border-radius:8px;padding:40px 48px;">
            {body_html}
          </td>
        </tr>
        <!-- footer -->
        <tr>
          <td align="center" style="padding:24px 0 0 0;">
            <span style="font-size:11px;color:{muted};">Ponder &middot; sdlc.threesix.ai</span>
          </td>
        </tr>
      </table>
    </td>
  </tr>
</table>
</body>
</html>"#,
        bg = theme::BG,
        primary = theme::PRIMARY,
        card = theme::CARD,
        card_border = theme::CARD_BORDER,
        muted = theme::MUTED,
        preheader = preheader,
        body_html = body_html,
    )
}
