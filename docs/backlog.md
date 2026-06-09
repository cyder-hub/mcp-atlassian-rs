# Backlog

This backlog records fixed future work that is outside the current Rust release. Each item has a fixed status, reason, current release behavior, unblock condition, and validation method.

## Auth And Token Flows

### OAuth Cloud 3LO Authorization-Code Flow

- Status: backlog.
- Reason: The current release implements BYOT access-token compatibility, not a browser authorization-code flow.
- Current Rust release behavior: Cloud Basic/API token, PAT-compatible request auth, and BYOT access tokens are supported; OAuth Cloud 3LO is not implemented.
- Unblock condition: define an OAuth app registration model, callback/listener behavior, consent flow, secure token handling, and user-facing setup docs.
- Required validation after implementation: run unit tests for callback parsing and token exchange, local integration tests with redaction, and real Cloud OAuth acceptance using disposable credentials.

### OAuth Proxy And DCR

- Status: backlog.
- Reason: Dynamic client registration and OAuth proxy behavior are not implemented in the current release.
- Current Rust release behavior: no OAuth proxy endpoint or DCR flow is exposed.
- Unblock condition: define proxy ownership, supported issuer behavior, client registration persistence, transport security, and failure handling.
- Required validation after implementation: run local proxy/DCR tests, reject unsafe redirect/client inputs, and verify tokens/secrets are redacted from logs and errors.

### OAuth Refresh And Token Storage

- Status: backlog.
- Reason: the current Rust release accepts provided access tokens but does not refresh or persist tokens.
- Current Rust release behavior: BYOT access tokens are loaded from env or request headers for the current process/request only.
- Unblock condition: define refresh-token storage, encryption or secret-manager integration, rotation behavior, expiry handling, and logout/revoke semantics.
- Required validation after implementation: run token refresh/expiry tests, storage redaction tests, and real refresh acceptance with disposable test credentials.

### Data Center OAuth Authorization-Code/Refresh

- Status: backlog.
- Reason: The current release keeps Server/Data Center auth to PAT or BYOT access-token fallback.
- Current Rust release behavior: Server/Data Center PAT is supported; Data Center OAuth authorization-code and refresh flows are not implemented.
- Unblock condition: define supported Data Center OAuth versions, app registration requirements, callback behavior, refresh semantics, and tenant compatibility matrix.
- Required validation after implementation: run unit/integration tests against a Data Center-compatible OAuth test service and record real acceptance on a disposable instance.

## Transport And Network

### SSE Transport

- Status: backlog.
- Reason: the Rust server currently supports stdio and streamable HTTP only.
- Current Rust release behavior: no SSE endpoint is exposed.
- Unblock condition: define whether SSE remains needed for target MCP clients, add server routing, lifecycle handling, auth behavior, and smoke scripts.
- Required validation after implementation: run local SSE initialize/tools/list/call smoke, request-auth regression tests if HTTP auth applies, and compatibility tests with the target MCP client.

### SOCKS Proxy

- Status: backlog.
- Reason: SOCKS proxy support is not compiled into the current HTTP client stack.
- Current Rust release behavior: no SOCKS-specific configuration surface is exposed.
- Unblock condition: choose a SOCKS-capable HTTP/TLS implementation path, define precedence with HTTP/HTTPS proxy and NO_PROXY, and preserve redaction of proxy credentials.
- Required validation after implementation: run proxy parsing tests, NO_PROXY tests, redaction tests, and mock outbound requests through a local SOCKS proxy.

### System Truststore Injection

- Status: backlog.
- Reason: The current release does not implement system truststore injection.
- Current Rust release behavior: service-specific TLS verification toggles and mTLS cert/key are supported.
- Unblock condition: choose a cross-platform truststore crate or OS-specific implementation and define interaction with custom TLS/mTLS settings.
- Required validation after implementation: run Linux truststore tests, failure-mode tests, and certificate-chain integration tests without leaking private key material.

### Encrypted mTLS Private Key Passwords

- Status: backlog.
- Reason: current mTLS support accepts PEM cert/key paths but not encrypted private key password envs.
- Current Rust release behavior: no encrypted-key password configuration surface is exposed.
- Unblock condition: define encrypted key parsing, password sourcing, redaction, and error behavior.
- Required validation after implementation: run encrypted-key unit tests, invalid-password tests, and redaction audits for password values.

## Release And Distribution

### Helm Chart

- Status: backlog.
- Reason: Helm packaging needs a Rust-specific chart contract and cannot be copied from an unrelated deployment model.
- Current Rust release behavior: no Helm chart is published or generated.
- Unblock condition: create a dedicated Helm execution document that defines the Rust image registry target, Kubernetes deployment contract, env and secret mapping, probes, ingress, HPA, RBAC, chart values, and validation commands.
- Required validation after implementation: render the chart, run chart lint, deploy to a test Kubernetes environment, confirm `/healthz`, confirm MCP streamable HTTP startup, and verify secret values do not appear in rendered public manifests.

### External Container Registry Publishing

- Status: backlog.
- Reason: The current release fixes reproducible local/CI Docker builds but does not publish to GHCR, Docker Hub, or another external registry.
- Current Rust release behavior: Docker image build is supported locally and in CI; no external image push is configured.
- Unblock condition: choose registry target, image naming, authentication, release tag policy, provenance/SBOM requirements, and retention policy.
- Required validation after implementation: build and push a release image from CI, pull it in a clean environment, run `/healthz`, and verify no secrets are embedded in image layers or logs.

### crates.io Publishing

- Status: backlog.
- Reason: Current release artifacts are GitHub Actions binary archives and checksums, not crates.io publication.
- Current Rust release behavior: no crates.io publish workflow is configured.
- Unblock condition: define crate ownership, package metadata, license/docs/readme readiness, semver policy, publish dry-run, and credential handling.
- Required validation after implementation: run `cargo package --locked`, inspect package contents, run publish dry-run where supported, and publish only from a protected release workflow.

## Real Acceptance Follow-Up

### Jira Service Management Real Acceptance

- Status: backlog.
- Reason: Service desk lookup returned 403 in the test tenant, so the implemented `jira_service_desks_read` toolset is local/product-dependency validated but not real-accepted.
- Current Rust release behavior: `jira_get_project_service_desk`, `jira_list_service_desk_queues`, and `jira_list_service_desk_queue_issues` are implemented with local mock coverage and documented as product/permission blocked for real acceptance.
- Unblock condition: provide a tenant, project, service desk, queue, and test identity with permission to read them.
- Required validation after implementation: run real acceptance for service desk lookup, queue listing, and queue issue listing without recording customer data or secrets.

### Jira Forms/ProForma Real Acceptance

- Status: backlog.
- Reason: Real acceptance did not receive an effective Forms API response and did not have a valid real form ID for details/update.
- Current Rust release behavior: `jira_list_issue_forms`, `jira_get_issue_form`, and `jira_update_issue_form_answers` are implemented with local mock coverage and documented as product/interface blocked for real acceptance.
- Unblock condition: provide a tenant with Jira Forms/ProForma enabled, a disposable test issue with a form, and a safe test answer update target.
- Required validation after implementation: run real form listing, form details, and safe answer update acceptance with cleanup and redacted compact errors.

### Confluence User Search Real Acceptance

- Status: backlog.
- Reason: Real acceptance did not execute a dedicated row for `confluence_search_users`.
- Current Rust release behavior: `confluence_search_users` is implemented with local mock coverage and documented as local-only.
- Unblock condition: add a dedicated real Confluence user-search acceptance row with a non-sensitive test query and stable expected result shape.
- Required validation after implementation: run the real user-search row and record only non-sensitive account/result metadata.

### Confluence Destructive Write Real Acceptance

- Status: backlog.
- Reason: Real acceptance validated only disabled-tool guards for `confluence_delete_page`, `confluence_move_page`, and `confluence_delete_attachment`; it did not execute destructive operations on real objects.
- Current Rust release behavior: these tools are implemented with local mock/disabled-tool coverage and documented as disabled-tool guard only for real acceptance.
- Unblock condition: provide disposable test pages and attachments with explicit isolation, cleanup, and permission to delete or move them.
- Required validation after implementation: run delete page, move page, and delete attachment acceptance against disposable objects only, then verify cleanup and absence of customer data in logs.

## Confluence Content Conversion

### Full md2conf Parity

- Status: backlog.
- Reason: The current release implements a deterministic minimal Markdown to Confluence storage conversion, not full `md2conf` feature parity.
- Current Rust release behavior: supported conversion covers headings, paragraphs, unordered lists, simple links, fenced code blocks, line breaks, and HTML escaping.
- Unblock condition: define the exact `md2conf` feature set to support, add fixtures for each supported construct, and decide whether a parser dependency is required.
- Required validation after implementation: run snapshot tests for all conversion fixtures, compare representative output against the defined feature contract, and verify no unsafe HTML is emitted.

### Mermaid And Macro Rendering

- Status: backlog.
- Reason: Mermaid and macro rendering are outside the current local Confluence loop.
- Current Rust release behavior: no Mermaid rendering, Confluence macro expansion, or macro-specific parity is claimed.
- Unblock condition: define supported macro types, storage format output, Mermaid rendering strategy, dependency/security review, and fixture coverage.
- Required validation after implementation: run conversion snapshots, Confluence create/update mock tests, and real acceptance on disposable pages with macro output inspection.
