# Protección de main — Contrato

## Branch Protection Rules (GitHub API)

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Check (stable)",
      "Check (MSRV 1.80)",
      "Format",
      "Clippy",
      "Test",
      "Doc",
      "Coverage"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "require_code_owner_reviews": true,
    "dismiss_stale_reviews": true
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": true
}
```

## Dependabot contract

| Ecosistema | Ignorar                        | Razón                         |
|------------|--------------------------------|-------------------------------|
| cargo      | nom >=8, toml >=1, chardetng >=1, tempfile >=3.20 | Breaking changes |
| github-actions | dtolnay/rust-toolchain >=1.81 | MSRV debe quedarse en 1.80 |

## CI requirements

| Job           | Toolchain | Comando                             |
|---------------|-----------|-------------------------------------|
| Check (stable)| stable    | `cargo check --all-features`        |
| Check (MSRV)  | 1.80      | `cargo check --all-features`        |
| Format        | stable    | `cargo fmt --all -- --check`        |
| Clippy        | stable    | `cargo clippy --all-features -- -D warnings` |
| Test          | stable    | `cargo test --all-features`         |
| Doc           | stable    | `cargo doc --no-deps --document-private-items` |
| Coverage      | stable    | `cargo tarpaulin --out Lcov --output-dir coverage` |

## Scripts de protección

```bash
# Aplicar branch protection a main
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --input branch-protection.json
```
