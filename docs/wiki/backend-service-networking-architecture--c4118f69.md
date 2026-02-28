<!--
source_wiki_id: c4118f69-ca7c-458b-b4fc-32cfa16179f0
source_system: governance-wiki-manager
domain_tag: infrastructure
team_tag: networking
product_tag: backend-service-networking
source_status: published
source_created_at: 2026-02-21 02:25:22.650556500
source_updated_at: 2026-02-21 23:39:17.452931500
migrated_at_utc: 2026-02-28T18:44:04+00:00
-->
# Backend Service Networking Architecture

`backend-service-networking` defines the library-mesh registration contract used for gateway service discovery.

## Responsibilities
- Define a versioned registry document mapping API contracts to internal services.
- Provide registry loading helpers for file and inline JSON configuration.
- Provide fallback single-service registry construction for local development.
- Validate that required contracts are registered before serving traffic.

## Registry Shape
- Top-level:
  - `version`
  - `services[]`
- Service entry:
  - `service_name`
  - `base_url`
  - `api_contracts[]`

## Current Contracts
- `worldbuilder.auth.register.v1`
- `worldbuilder.auth.login.v1`
- `worldbuilder.auth.refresh.v1`
- `worldbuilder.auth.guest-upgrade.v1`
- `worldbuilder.discovery.catalog.v1`
- `worldbuilder.discovery.detail.v1`
- `worldbuilder.discovery.schema.v1`
- `worldbuilder.discovery.play-session.get.v1`
- `worldbuilder.discovery.home_feed.v1`
- `worldbuilder.discovery.publish.create.v1`

## Runtime Configuration
- `WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH`
- `WORLD_BUILDER_SERVICE_MESH_REGISTRY_JSON`

Loading precedence:
1. inline JSON env var
2. path-based JSON file
3. caller-provided local fallback single-service registry

## K8s/GCP Wiring Pattern
- Store registry JSON in a ConfigMap.
- Mount registry file in gateway pods.
- Set `WORLD_BUILDER_SERVICE_MESH_REGISTRY_PATH` to mounted path.

This preserves sidecar-free, Rust-library mesh behavior while staying deployable on GCP Kubernetes.
