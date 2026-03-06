ALTER TABLE orchestrator_webhook_routes ADD COLUMN IF NOT EXISTS secret_token TEXT;
ALTER TABLE orchestrator_webhook_routes ADD COLUMN IF NOT EXISTS store_only BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX IF NOT EXISTS orchestrator_webhooks_route_received_idx
    ON orchestrator_webhooks (route_path, received_at);
