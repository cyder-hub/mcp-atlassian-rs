## Summary

-

## Verification

Commands run:

- [ ] `cargo fmt --check`
- [ ] `RUSTFLAGS="-Dwarnings" cargo check`
- [ ] `cargo test`
- [ ] `cargo build --release`
- [ ] `cargo xtask smoke jira stdio`
- [ ] `cargo xtask smoke jira http`
- [ ] `cargo xtask smoke jira restricted`
- [ ] `cargo xtask smoke confluence all`
- [ ] `docker compose -f docker-compose.yml config`
- [ ] `docker build -t mcp-atlassian-rs:ci -f Dockerfile .`
- [ ] Container `/healthz` smoke
- [ ] Not run; reason:

## Impact

- [ ] Rust service code
- [ ] Docker or compose
- [ ] GitHub Actions or repository metadata
- [ ] Documentation only
- [ ] Release artifact, changelog, or version policy
- [ ] Support matrix or backlog
- [ ] Dependencies or lockfiles

## Checklist

- [ ] I did not commit `target/`, `.env`, logs, or credentials.
- [ ] I updated README, contributing guidance, support matrix, backlog, or release notes where behavior, commands, naming, support status, or dependencies changed.
- [ ] I avoided adding claims for unimplemented features.
