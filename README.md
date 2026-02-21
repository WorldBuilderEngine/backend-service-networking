# backend-service-networking

Library mesh registration contract for backend service API discovery.

## Purpose
- Define a stable API contract registry document that maps API contracts to internal services.
- Let `backend-gateway` resolve contracts through this library instead of hardcoding service routes.
- Keep sidecar-free service mesh integration by using one shared Rust library contract.

## Registry Document
```json
{
  "version": "2026-02-21",
  "services": [
    {
      "service_name": "backend-data-center",
      "base_url": "http://backend-data-center.infrastructure.svc.cluster.local:8787",
      "api_contracts": [
        "worldbuilder.discovery.catalog.v1",
        "worldbuilder.discovery.home.v1",
        "worldbuilder.discovery.detail.v1",
        "worldbuilder.discovery.schema.v1",
        "worldbuilder.discovery.play-session.get.v1",
        "worldbuilder.discovery.play-session.create.v1"
      ]
    }
  ]
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

## GCP K8s Wiring
- Store registry JSON in a ConfigMap and mount as file.
- Point gateway env var to that mounted file path.

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
