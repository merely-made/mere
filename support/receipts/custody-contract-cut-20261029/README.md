# Custody contract cut, 2026-10-29

`RemoteAuthRevocationOutcome` now belongs to `personae::carry`, beside `DeviceId`.
Pandect no longer exports either type. Consumers must import their owner directly;
there are no compatibility aliases. Persistent revocation and authority behavior
are unchanged.

Mere consumers, tests and examples migrate together. Signalman desktop migrates
its four callers and drops its direct Pandect dependency. The endpoint door no
longer pulls Pandect, the graph kernel, Alembic or the Graphshell application
through its dependency closure. It still contains projection and custody client
machinery; separating those is the next, separately reviewed half.

`qualification.json` records focused checks, exact source commits, log hashes,
resource limits and unresolved qualification boundaries. This is Git source
publication, not a crates.io release or full product qualification.
