# Application projection offers

## Goal and authority

An application can expose its existing projection endpoint through Djinn without an application-specific source branch or route-table entry. Mark approved this next step after the custody/projection separation. The application owns projection implementation and lifetime; Djinn owns provider permission, viewer admission and routing. An offer grants neither custody privileges nor viewer access.

## Architecture

Use the existing application door and shared AppConnection handshake. A reserved projection-provider route switches the existing typed Call/Answer slots to a concrete provider protocol. Existing custody and viewer frames remain byte-compatible. Do not add a broker registry, retry framework, address-based reverse connection, or package identity.

A provider declares one route and label, then pulls bounded work for its endpoint instances. The host supplies a session identifier and narrow, host-asserted viewer context (session, subject, application), never delegation credentials. The provider opens an instance per viewer, executes existing CarrierRequest operations and returns typed replies/notices. Provider work IDs and session IDs are checked. Viewer Open/Close/Suspend and retained authority remain host responsibilities.

Djinn permits configured application/route pairs separately from viewer grants. Defaults admit no external provider. Built-in routes cannot be replaced. A generation-bound lease withdraws on disconnect, explicit withdrawal, owner revocation, timeout or protocol failure. Withdrawal prevents new work, invalidates active sessions, wakes waiting callers and discards late results. Already-dispatched application side effects cannot be rolled back.

Preserve the existing synchronous ResidentEndpoint boundary with a cheap proxy factory. Never wait for provider traffic under the catalog lock or on Tokio runtime workers. Offered sessions run on bounded blocking workers with finite cancellation-aware waits. Initialize the provider instance before acknowledging the viewer connection and cache its descriptor. Notice intake is bounded/coalesced. Provider frame reading remains continuously polled; partial native frames are never discarded and then resumed from the wrong boundary.

## Modules and dependencies

- graphshell-endpoint: portable `door::projection_offer` DTOs behind `projection-offer`; native `door::projection_provider` client/runner behind `projection-provider`, reusing AppConnection. Neither feature activates custody.
- graphshell-host: narrow lifecycle/catalog additions needed for leased endpoint preparation/removal, preserving existing local consumers.
- Djinn `door::projection_offers`: owner policy, leases, bounded work queues and proxy endpoint; app_broker handles provider admission through the existing handshake.
- Djinn owner settings: explicit provider route permissions, viewer labels and bounded limits. Applying/replacing policy revokes affected leases. Supported resident startup loads the policy; library hosts use the same explicit setter.
- Knot `knot_endpoint`: native provider mode for its real directory endpoint. Existing stdio modes remain distinct supported transports; no aliases delaying a breaking migration.

Portable DTOs remain Wasm-checkable. Native serving remains Unix/Windows only; Windows runtime qualification requires its own executor receipts.

## Work and done conditions

1. Freeze concrete provider DTO wire fixtures and rejection tests; implement native provider runner with independent custody-free consumer fixture.
2. Add host policy and lease behavior with tests for unauthorized registration, reserved/duplicate route, stale cleanup, disconnect, revoke, pending/in-flight work and bounded resources. Keep the existing authority loop and custody-while-locked behavior.
3. Integrate owner settings and the actual Knot standalone endpoint. Prove a fresh application label/route can serve a real projection without a Djinn source entry, and a viewer still needs its own grant.
4. Run bounded endpoint/host/Djinn consuming targets, isolated native/Wasm feature fixtures and Knot pilot tests. Review source and lock dependency edges. Retain exact receipts and limitations, then publish supplier before consumer directly to main.

## Global constraints and review focus

Check upstream before basing and publishing; migrate every changed API consumer immediately. Preserve 1 MiB native-endian framing, existing socket/env/schema names and safe error diagnostics/owned frame wiping. Same-user AppId labels are not executable authentication. Keep app-to-Djinn distinct from Djinn-to-custodian custody-wire. No broad supplier refresh, no extraction of embedded Knot's background duties. Heavy builds use the existing machine-heavy lock, 3 GiB memory, zero swap, 150% CPU and one Cargo job. Review lease races, idle disconnect, frame cancellation, error redaction, session isolation, and runtime blocking explicitly.
