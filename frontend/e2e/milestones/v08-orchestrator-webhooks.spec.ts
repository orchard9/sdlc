/**
 * E2E spec for milestone: v08-orchestrator-webhooks
 * "Webhook ingestion and routing — external triggers fire tools on next tick"
 *
 * Mixed-mode spec:
 *  - API tests: POST/GET /api/orchestrator/webhooks/routes, POST /webhooks/*
 *  - Source verification: webhook data models, DB tables, route wiring
 *
 * No UI exists for webhooks — all scenarios exercise the REST layer directly.
 * Locator policy: request fixture for API; fs for source verification.
 */

import { test, expect } from '@playwright/test'
import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const PROJECT_ROOT = path.resolve(__dirname, '../../../')

// ---------------------------------------------------------------------------
// Scenario 1: Register a webhook route
// POST /api/orchestrator/webhooks/routes → 201 + route appears in GET list
// ---------------------------------------------------------------------------

test('POST /api/orchestrator/webhooks/routes returns 201 with route object', async ({ request }) => {
  const slug = `/test-uat-route-${Date.now()}`
  const res = await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: slug,
      tool_name: 'quality-check',
      input_template: '{}',
    },
  })
  expect(res.status(), `Expected 201, got ${res.status()}`).toBe(201)
  const body = await res.json()
  expect(body).toHaveProperty('id')
  expect(body.path).toBe(slug)
  expect(body.tool_name).toBe('quality-check')
  expect(body.input_template).toBe('{}')
})

test('registered route appears in GET /api/orchestrator/webhooks/routes', async ({ request }) => {
  const slug = `/test-uat-list-${Date.now()}`
  // Register first
  const reg = await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: slug,
      tool_name: 'quality-check',
      input_template: '{}',
    },
  })
  expect(reg.status()).toBe(201)

  // Now list routes — registered route must appear
  const listRes = await request.get('/api/orchestrator/webhooks/routes')
  expect(listRes.ok(), `Expected 200, got ${listRes.status()}`).toBeTruthy()
  const routes = await listRes.json()
  expect(Array.isArray(routes)).toBe(true)
  const found = routes.find((r: { path: string }) => r.path === slug)
  expect(found, `Route ${slug} not found in list`).toBeTruthy()
  expect(found.tool_name).toBe('quality-check')
})

// ---------------------------------------------------------------------------
// Scenario 2: Fire a webhook and receive 202
// The storage layer must accept the payload regardless of daemon running
// ---------------------------------------------------------------------------

test('POST /webhooks/<registered-path> returns 202 with payload id', async ({ request }) => {
  const routePath = `/test-uat-fire-${Date.now()}`
  // Register the route first
  await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: routePath,
      tool_name: 'quality-check',
      input_template: '{}',
    },
  })

  // Fire the webhook — strip leading slash for URL segment
  const urlSegment = routePath.slice(1)
  const res = await request.post(`/webhooks/${urlSegment}`, {
    data: { event: 'push', ref: 'main' },
    headers: { 'Content-Type': 'application/json' },
  })
  expect(res.status(), `Expected 202, got ${res.status()}`).toBe(202)
  const body = await res.json()
  expect(body).toHaveProperty('id')
  // id should be a non-empty UUID string
  expect(typeof body.id).toBe('string')
  expect(body.id.length).toBeGreaterThan(10)
})

// ---------------------------------------------------------------------------
// Scenario 3: Unregistered webhook route returns 202 (stored, not crashed)
// ---------------------------------------------------------------------------

test('POST /webhooks/unknown-route returns 202 without crash', async ({ request }) => {
  const res = await request.post('/webhooks/unregistered-route-that-does-not-exist', {
    data: {},
    headers: { 'Content-Type': 'application/json' },
  })
  // Must return 202, not 404 or 5xx — payload is stored, routing is deferred
  expect(res.status(), `Expected 202 for unregistered route, got ${res.status()}`).toBe(202)
  const body = await res.json()
  expect(body).toHaveProperty('id')
})

// ---------------------------------------------------------------------------
// Scenario 4: Payload mapping — input_template with {{payload}} placeholder
// ---------------------------------------------------------------------------

test('registered route with {{payload}} template stores correctly (201)', async ({ request }) => {
  const slug = `/test-uat-template-${Date.now()}`
  const template = '{"source":"webhook","body":{{payload}}}'
  const res = await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: slug,
      tool_name: 'quality-check',
      input_template: template,
    },
  })
  expect(res.status()).toBe(201)
  const body = await res.json()
  expect(body.input_template).toBe(template)
})

test('POST /webhooks/<template-route> returns 202 with payload id (mapping stored)', async ({
  request,
}) => {
  const routePath = `/test-uat-map-${Date.now()}`
  await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: routePath,
      tool_name: 'quality-check',
      input_template: '{"source":"webhook","body":{{payload}}}',
    },
  })

  const urlSegment = routePath.slice(1)
  const res = await request.post(`/webhooks/${urlSegment}`, {
    data: { key: 'value' },
    headers: { 'Content-Type': 'application/json' },
  })
  expect(res.status()).toBe(202)
  const body = await res.json()
  expect(body).toHaveProperty('id')
})

// ---------------------------------------------------------------------------
// Source verification: WebhookPayload and WebhookRoute exist in core
// ---------------------------------------------------------------------------

test('orchestrator/webhook.rs defines WebhookPayload with id, route_path, raw_body', () => {
  const webhookPath = path.join(PROJECT_ROOT, 'crates/sdlc-core/src/orchestrator/webhook.rs')
  expect(fs.existsSync(webhookPath), 'webhook.rs must exist').toBe(true)
  const src = fs.readFileSync(webhookPath, 'utf8')
  expect(src).toContain('pub struct WebhookPayload')
  expect(src).toMatch(/pub id:\s*Uuid/)
  expect(src).toMatch(/pub route_path:\s*String/)
  expect(src).toMatch(/pub raw_body:\s*Vec<u8>/)
})

test('orchestrator/webhook.rs defines WebhookRoute with render_input method', () => {
  const webhookPath = path.join(PROJECT_ROOT, 'crates/sdlc-core/src/orchestrator/webhook.rs')
  const src = fs.readFileSync(webhookPath, 'utf8')
  expect(src).toContain('pub struct WebhookRoute')
  expect(src).toMatch(/pub path:\s*String/)
  expect(src).toMatch(/pub tool_name:\s*String/)
  expect(src).toMatch(/pub input_template:\s*String/)
  expect(src).toContain('pub fn render_input')
  // Template substitution uses {{payload}}
  expect(src).toContain('{{payload}}')
})

// ---------------------------------------------------------------------------
// Source verification: DB tables for WEBHOOKS and WEBHOOK_ROUTES
// ---------------------------------------------------------------------------

test('orchestrator/db.rs defines WEBHOOKS and WEBHOOK_ROUTES redb tables', () => {
  const dbPath = path.join(PROJECT_ROOT, 'crates/sdlc-core/src/orchestrator/db.rs')
  expect(fs.existsSync(dbPath), 'db.rs must exist').toBe(true)
  const src = fs.readFileSync(dbPath, 'utf8')
  expect(src).toMatch(/TableDefinition.*webhooks/)
  expect(src).toMatch(/TableDefinition.*webhook_routes/)
  expect(src).toContain('insert_webhook')
  expect(src).toContain('all_pending_webhooks')
  expect(src).toContain('insert_route')
  expect(src).toContain('list_routes')
})

// ---------------------------------------------------------------------------
// Source verification: routes wired in lib.rs
// ---------------------------------------------------------------------------

test('sdlc-server lib.rs wires POST /webhooks/{route} and webhook route management', () => {
  const libPath = path.join(PROJECT_ROOT, 'crates/sdlc-server/src/lib.rs')
  expect(fs.existsSync(libPath), 'lib.rs must exist').toBe(true)
  const src = fs.readFileSync(libPath, 'utf8')
  // Webhook ingestion wildcard route
  expect(src).toMatch(/\/webhooks\/\{route\}|\/webhooks\/\*|receive_webhook/)
  // Route management endpoint
  expect(src).toMatch(/\/api\/orchestrator\/webhooks\/routes/)
  expect(src).toContain('register_route')
  expect(src).toContain('list_routes')
})

// ---------------------------------------------------------------------------
// API validation: bad inputs return 400
// ---------------------------------------------------------------------------

test('POST /api/orchestrator/webhooks/routes with missing path returns 400', async ({ request }) => {
  const res = await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: 'no-leading-slash',
      tool_name: 'quality-check',
      input_template: '{}',
    },
  })
  expect(res.status()).toBe(400)
})

test('POST /api/orchestrator/webhooks/routes with empty tool_name returns 400', async ({
  request,
}) => {
  const res = await request.post('/api/orchestrator/webhooks/routes', {
    data: {
      path: `/test-empty-tool-${Date.now()}`,
      tool_name: '',
      input_template: '{}',
    },
  })
  expect(res.status()).toBe(400)
})

test('GET /api/orchestrator/webhooks/routes returns JSON array', async ({ request }) => {
  const res = await request.get('/api/orchestrator/webhooks/routes')
  expect(res.ok(), `Expected 200, got ${res.status()}`).toBeTruthy()
  const body = await res.json()
  expect(Array.isArray(body)).toBe(true)
})
