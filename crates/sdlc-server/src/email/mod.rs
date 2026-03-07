//! Themed email rendering for Ponder.
//!
//! Provides `render_otp()` which returns a fully-themed HTML email with all
//! CSS inlined (no `<style>` blocks) so email clients cannot apply dark-mode
//! inversion or system defaults.

pub mod layout;
pub mod theme;

/// Pre-rendered email content ready to send via the notify service.
pub struct EmailContent {
    pub subject: String,
    pub html: String,
    pub text: String,
}

/// Render the OTP invite email for `email` with the given `otp` code.
pub fn render_otp(email: &str, otp: &str) -> EmailContent {
    let subject = format!("Your Ponder access code: {otp}");

    let body_html = format!(
        r#"<table width="100%" cellpadding="0" cellspacing="0" border="0">
  <tr>
    <td style="padding:0 0 8px 0;">
      <span style="font-size:20px;font-weight:600;color:{fg};">You've been invited</span>
    </td>
  </tr>
  <tr>
    <td style="padding:0 0 28px 0;">
      <span style="font-size:14px;color:{muted};">Enter this code to access Ponder. It expires in 48 hours.</span>
    </td>
  </tr>
  <!-- OTP code box -->
  <tr>
    <td align="center" style="padding:0 0 24px 0;">
      <table cellpadding="0" cellspacing="0" border="0" bgcolor="{bg}" style="background-color:{bg};border-radius:6px;">
        <tr>
          <td style="padding:20px 40px;">
            <span style="font-size:36px;font-weight:700;letter-spacing:8px;color:{primary};font-family:monospace;">{otp}</span>
          </td>
        </tr>
      </table>
    </td>
  </tr>
  <tr>
    <td style="padding:0 0 8px 0;">
      <span style="font-size:12px;color:{muted};">Expires in 48 hours. Do not share this code.</span>
    </td>
  </tr>
  <tr>
    <td>
      <span style="font-size:12px;color:{muted};">If you didn't request access to Ponder, you can safely ignore this email.</span>
    </td>
  </tr>
</table>"#,
        fg = theme::FG,
        muted = theme::MUTED,
        bg = theme::BG,
        primary = theme::PRIMARY,
        otp = otp,
    );

    let html = layout::wrap(
        &format!("Your Ponder access code for {email} is {otp}"),
        &body_html,
    );

    let text = format!(
        "You've been invited to Ponder.\n\nYour access code is: {otp}\n\nThis code expires in 48 hours. Do not share it.\n\nIf you didn't request access, you can safely ignore this email.\n\n---\nPonder · sdlc.threesix.ai",
        otp = otp,
    );

    EmailContent {
        subject,
        html,
        text,
    }
}
