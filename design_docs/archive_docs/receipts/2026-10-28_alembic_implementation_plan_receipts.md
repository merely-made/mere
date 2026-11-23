# Alembic implementation receipts

## 2026-11-23. Native consumers of the schema-2 workshop

Both runs used macOS 26.5.1 on an arm64 M4 and Rust 1.99.1. Cargo resolved
`mere-athanor` to Distillery `9321345b479568d2f1964d826bb7d56bd8c1908f`.
The generated lockfile SHA-256 was
`efc61285b69ba4e0e0924867c4b38934d582b3284db55f8db7a57ceb46d70624`.
The local receipt directory is `Code/output/decision-driver-hosts-20261123/`.

| Consumer | Turnstone commit | Entry point | Observation |
|---|---|---|---|
| Graphshell | `b365e0cd4e14cb76d46b35104ece960d8c95edb8` | `graphshell-alembic` via the `alembic_workshop_process` integration test and `--features alembic-workshop` | Four native processes generated, accepted, inspected and reconciled. Node counts were 3, 4, 4 and 4. Cold reopen retained the same proposal and receipt with zero model turns. |
| Turnstone | `da3706de025aa47e74881b07b9445f89e0721e02` | `test -p turnstone --lib --features workshop alembic_workshop` and `build -p turnstone --bin turnstone --features workshop` | Nine tests passed. A fresh child process recovered the original public proposal and receipt after loss of the destination acknowledgement. The product binary built. |

Graphshell refused an archived schema-1 run. The SHA-256 before and after was
`d9db094a157d50225692878e69ab3dd2404a564b5dc1449d981658b66803f80e`.
Turnstone refused a sealed schema-1 fixture under synthetic protection with
unchanged storage bytes. These process and host-adapter results cover the
D18 compatibility checks. W3's full matrix, headed controls and production
custody have separate qualification.
