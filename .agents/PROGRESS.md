Initial deployment to kaizen.tallisa.dev
- GitOps manifests in cumulus-gitops: namespace, deployment, service, httproute, db-init job, network policy, image automation, externalsecrets
- Bitwarden secrets created under K8s project (305a7f76): DATABASE_URL, JWT_SECRET, KAIZEN_DB_PASSWORD, POSTGRES_PASSWORD, TIMESCALE_HOST, registry dockerconfigjson
- DB user: kaizen, DB name: kaizen, on shared timescale instance
- Timescale network policy updated to allow kaizen namespace
- CI workflow: .github/workflows/kaizen.yml (build-and-push only, no test job yet)
- TRADEOFF: No test job in CI since there are no tests written yet. Add when tests exist.
- TODO: Dockerfile EXPOSE should be 9081 not 8080 to match forge.toml gateway port
