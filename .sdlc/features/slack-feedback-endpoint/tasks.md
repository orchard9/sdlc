# Tasks: slack-feedback-endpoint

## Task List

1. **Add SlackFeedbackPayload struct and receive_slack_feedback handler** — Add the deserialization structs (`SlackFeedbackPayload`, `SlackContextMessage`) and the `receive_slack_feedback` async handler to `crates/sdlc-server/src/routes/feedback.rs`. Include validation (source must be "slack", text and user_name required), context markdown rendering, dedup check via body marker, thread creation via `sdlc_core::feedback_thread::create_thread`, first post via `add_post`, and 201 response.

2. **Register route in lib.rs** — Add `POST /api/feedback/slack` to the router in `crates/sdlc-server/src/lib.rs`, mapping to `feedback::receive_slack_feedback`.

3. **Add unit tests** — Add tests in `feedback.rs` covering: valid payload creates thread (201), missing text returns 400, missing user_name returns 400, wrong source returns 400, duplicate message_ts returns 409, payload with no context_messages works, payload with context_messages renders markdown body correctly.
