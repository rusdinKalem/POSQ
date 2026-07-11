# ADR-0011: Device-Bound License and Zero-Trust Local-Server Integration

Date: 2026-07-05  
Status: Accepted

## Context

The server issues license tokens to the desktop POS. A signed token alone proves the token was issued by the server, but does not prove it is being used on the original activated device. Without device binding, a token copied from one PC may be reused on another PC until expiration.

The control plane also exposes high-risk multi-tenant APIs. API security guidance emphasizes object-level authorization, authentication correctness, function-level authorization, resource limits, and security configuration. The POS architecture must assume no implicit trust based on network location.

## Decision

Use device-bound license and zero-trust integration between desktop and server.

Required:

- Desktop generates per-install `install_id`.
- Desktop generates device key pair during activation.
- Device private key stays local in OS secure storage where available.
- Server stores device public key/thumbprint.
- Activation and heartbeat use challenge-response or signed request body.
- License token includes device-bound claims.
- Desktop verifies issuer, audience, algorithm, key id, expiry, not-before, token version, merchant id, device id, install id hash, public key thumbprint, and signature.
- Server enforces tenant/object authorization on every merchant-owned resource.
- Server treats body `merchant_id` as untrusted until matched with auth context.

## Consequences

Positive:

- Copied license token is not enough to activate another PC.
- Heartbeat replay is detectable.
- Tenant isolation becomes explicit.
- Server and desktop have a stronger trust contract.

Tradeoffs:

- Reactivation flow is needed if local secure storage is lost.
- Device replacement support must be handled in admin dashboard.
- Implementation requires crypto tests and compatibility tests.

## Implementation References

- `SERVER_LOCAL_INTEGRATION_SECURITY.md`
- `SERVER_HARDENING_REVIEW.md`
- `SERVER_API_WORKFLOWS.md`
- `SERVER_DATA_MODEL.md`
- `SERVER_TEST_PLAN.md`
- `LICENSE_LIFECYCLE.md`
- `SECURITY_MODEL.md`

