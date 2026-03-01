# Security Audit: data-testid Attributes

## Scope

This audit covers the addition of `data-testid` HTML attributes to React components and the introduction of a `testId` prop on the `StatusBadge` component.

## Security Analysis

### 1. Data exposure via HTML attributes

`data-testid` values are hard-coded string literals (`"phase-badge"`, `"feature-title"`, etc.). They carry no user data, no secrets, and no server-side identifiers. An attacker inspecting the DOM learns only that these elements exist and what semantic role they play — information already visible from the element's content and structure.

**Finding:** No information disclosure risk. No action required.

### 2. Injection through the `testId` prop

The `testId` prop flows directly into `data-testid={testId}`. React's JSX renderer escapes attribute values before writing them to the DOM, so no XSS vector exists here regardless of what string is passed. All current call sites pass string literals; no prop is derived from user input or server responses.

**Finding:** No injection risk. No action required.

### 3. Test ID enumeration / selector stability as an attack surface

`data-testid` attributes are used by automated test frameworks. Knowing these selectors does not grant any privilege. They do not represent API endpoints, authentication tokens, or CSRF anchors.

**Finding:** No exploitable attack surface. No action required.

### 4. Supply-chain / dependency risk

No new npm packages were added. The change is entirely in TypeScript source files. Build output is the same React bundle with additional inert HTML attributes.

**Finding:** No supply-chain impact.

### 5. Regression / information-hiding concern (frontend only)

The attributes are present in production builds (not gated behind a `NODE_ENV` check). This is intentional and standard practice — hiding `data-testid` in production adds no security while breaking headless tooling used for monitoring and accessibility.

**Finding:** Acceptable. No remediation needed.

## Verdict

**Pass.** This change introduces no security concerns. It adds static, hard-coded HTML attribute strings with no user-controlled values and no new data exposure. The `testId` prop is type-safe and React-escaped. No remediation required.
