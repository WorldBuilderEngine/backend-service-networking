# Publish Ingress Policy Contract

## Objective
Define one source of truth for publish ingress payload limits across:
- `backend-edge`
- `backend-gateway`
- `backend-data-center`

Owner product: `backend-service-networking`

## Shared Contract
- Registry key: `publish_ingress_policy`
- Publish API contract: `worldbuilder.discovery.publish.create.v1`
- Canonical minimum limit: `67108864` bytes (`64 MiB`)
- Required rollout invariant: no hop may be configured below this minimum

Required hop mappings:
- `backend-edge` -> `WORLD_BUILDER_EDGE_MAX_JSON_BODY_BYTES`
- `backend-gateway` -> `WORLD_BUILDER_APOLLO_MAX_JSON_BODY_BYTES`
- `backend-data-center` -> `WORLD_BUILDER_DATA_CENTER_MAX_JSON_BODY_BYTES`

## Conformance Checks
Use `ServiceMeshRegistry` guardrails:
- Startup: `ensure_publish_ingress_hop_limit_from_environment(hop_name)`
- CI/Deploy: `ensure_publish_ingress_all_hops_conform(...)`

Both checks fail fast when any hop value is below the policy.

## Observability
Counter metric:
- `worldbuilder_publish_ingress_payload_rejected_total`

Required dimensions/log fields:
- `publishIngressHop`
- `configuredMaxBodyBytes`
- `requiredPolicyBytes`
- `requestContentLength`
- `requestId`
- `apiContract`
