# backend-service-networking

Library mesh registration contract for backend service API discovery.

## Purpose
- Define a stable API contract registry document that maps API contracts to internal services.
- Let `backend-gateway` resolve contracts through this library instead of hardcoding service routes.
- Keep sidecar-free service mesh integration by using one shared Rust library contract.

## Registry Document
```json
{
  "version": "2026-03-01",
  "services": [
    {
      "service_name": "backend-discovery-home",
      "base_url": "http://backend-discovery-home.discovery.svc.cluster.local:8790",
      "api_contracts": [
        "worldbuilder.discovery.home_feed.v1"
      ]
    },
    {
      "service_name": "backend-data-center",
      "base_url": "http://backend-data-center.infrastructure.svc.cluster.local:8790",
      "api_contracts": [
        "worldbuilder.discovery.catalog.v1",
        "worldbuilder.discovery.detail.v1",
        "worldbuilder.discovery.schema.v1",
        "worldbuilder.discovery.play-session.get.v1",
        "worldbuilder.discovery.publish.create.v1"
      ]
    },
    {
      "service_name": "backend-auth",
      "base_url": "http://backend-auth.infrastructure.svc.cluster.local:8791",
      "api_contracts": [
        "worldbuilder.auth.register.v1",
        "worldbuilder.auth.login.v1",
        "worldbuilder.auth.refresh.v1",
        "worldbuilder.auth.guest-upgrade.v1"
      ]
    },
    {
      "service_name": "backend-accounts",
      "base_url": "http://backend-accounts.infrastructure.svc.cluster.local:8787",
      "api_contracts": [
        "worldbuilder.accounts.internal-bootstrap.v1",
        "worldbuilder.accounts.get-by-id.v1",
        "worldbuilder.accounts.get-by-identity.v1",
        "worldbuilder.accounts.update.v1"
      ]
    },
    {
      "service_name": "backend-identity",
      "base_url": "http://backend-identity.infrastructure.svc.cluster.local:8786",
      "api_contracts": [
        "worldbuilder.identity.profile.upsert.v1",
        "worldbuilder.identity.profile.get.v1",
        "worldbuilder.identity.policy-evaluation.v1"
      ]
    }
  ],
  "publish_ingress_policy": {
    "policy_owner_product": "backend-service-networking",
    "publish_api_contract": "worldbuilder.discovery.publish.create.v1",
    "default_max_body_bytes": 67108864,
    "required_hops": [
      {
        "hop_name": "backend-edge",
        "product": "backend-edge",
        "max_body_bytes_env_var": "WORLD_BUILDER_EDGE_MAX_JSON_BODY_BYTES"
      },
      {
        "hop_name": "backend-gateway",
        "product": "backend-gateway",
        "max_body_bytes_env_var": "WORLD_BUILDER_APOLLO_MAX_JSON_BODY_BYTES"
      },
      {
        "hop_name": "backend-data-center",
        "product": "backend-data-center",
        "max_body_bytes_env_var": "WORLD_BUILDER_DATA_CENTER_MAX_JSON_BODY_BYTES"
      }
    ],
    "observability": {
      "rejection_metric_name": "worldbuilder_publish_ingress_payload_rejected_total",
      "rejection_log_fields": [
        "publishIngressHop",
        "configuredMaxBodyBytes",
        "requiredPolicyBytes",
        "requestContentLength",
        "requestId",
        "apiContract"
      ]
    }
  }
}
```

## Local/Dev Wiring
- Provide one of:
  - `WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH` to a JSON file.
  - `WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON` as inline JSON.
- If neither is set, callers can fallback to a single-service registry built from local upstream settings.
- Runtime loading behavior in this crate:
  - `ServiceMeshRegistry::from_environment()` checks `WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON` first, then `WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH`.
  - `ServiceMeshRegistry::from_environment_or_single_service(...)` loads from env when configured, else builds the provided fallback single-service registry.
  - `ServiceMeshRegistry::ensure_contracts_registered(...)` verifies required contracts are present before serving traffic.
  - `ServiceMeshRegistry::ensure_publish_ingress_hop_limit_from_environment(hop_name)` verifies each hop's configured env max-body-bytes is not below the shared policy.
  - `ServiceMeshRegistry::ensure_publish_ingress_all_hops_conform(...)` verifies all edge/gateway/data-center limits conform in CI/deploy checks.

## Publish Ingress Policy Contract
- Owner: `backend-service-networking`.
- Contract key: `publish_ingress_policy`.
- Canonical default: `67108864` bytes (`64 MiB`) across all publish ingress hops.
- Required rollout invariant: no hop can run lower than `default_max_body_bytes`.
- Required hops:
  - `backend-edge` via `WORLD_BUILDER_EDGE_MAX_JSON_BODY_BYTES`
  - `backend-gateway` via `WORLD_BUILDER_APOLLO_MAX_JSON_BODY_BYTES`
  - `backend-data-center` via `WORLD_BUILDER_DATA_CENTER_MAX_JSON_BODY_BYTES`

If a hop is below policy, this crate raises a startup/validation error so drift is blocked before publish traffic is served.

## Observability Contract
- Counter metric: `worldbuilder_publish_ingress_payload_rejected_total`
- Required dimensions/log fields:
  - `publishIngressHop`
  - `configuredMaxBodyBytes`
  - `requiredPolicyBytes`
  - `requestContentLength`
  - `requestId`
  - `apiContract`

## GCP K8s Wiring
- Store registry JSON in a ConfigMap and mount as file.
- Point gateway env var to that mounted file path.
- Ready-to-apply examples in this repository:
  - `deploy/k8s/registry.json`
  - `deploy/k8s/backend-service-networking-registry.configmap.yaml`
  - `deploy/k8s/backend-gateway-registry.patch.yaml`

Example deployment environment:
```yaml
env:
  - name: WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH
    value: /etc/worldbuilder/mesh/registry.json
volumeMounts:
  - name: mesh-registry
    mountPath: /etc/worldbuilder/mesh
    readOnly: true
volumes:
  - name: mesh-registry
    configMap:
      name: backend-service-networking-registry
```
