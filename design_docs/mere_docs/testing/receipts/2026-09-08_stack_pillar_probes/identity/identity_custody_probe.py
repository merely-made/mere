"""Disposable R2-B/C model probe.

This does not implement Turnstone contracts.  It tests the identity and reference
rules proposed in the family composition research brief using standard-library
types and an in-memory store.  SHA-256 stands in for Muniment's BLAKE3 because
the experiment concerns key equality and reference behavior, not hash choice.
"""

from __future__ import annotations

from dataclasses import dataclass, replace
import hashlib
import itertools
import json
import unittest


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


@dataclass(frozen=True)
class CaptureTarget:
    session: str
    node: str
    document_generation: int
    surface: str


@dataclass(frozen=True)
class PendingCapture:
    request: int
    target: CaptureTarget
    source_address: str


@dataclass(frozen=True)
class CaptureObservation:
    observation: str
    request: int
    target: CaptureTarget
    source_address: str
    artifact_hash: str


class CaptureRegistry:
    def __init__(self) -> None:
        self.pending: dict[tuple[str, int], PendingCapture] = {}
        # Request values are single-use for one surface instance.  A real host
        # can implement this with a non-wrapping allocator or put an epoch in
        # the surface identity; this model deliberately assumes neither exists
        # in production yet.
        self.seen_requests: set[tuple[str, int]] = set()
        self.observations: dict[str, CaptureObservation] = {}
        self.blobs: dict[str, bytes] = {}

    def start(self, pending: PendingCapture) -> None:
        key = (pending.target.surface, pending.request)
        if key in self.seen_requests:
            raise ValueError("reused request identity")
        self.seen_requests.add(key)
        self.pending[key] = pending

    def complete(
        self, surface: str, request: int, live_target: CaptureTarget, data: bytes
    ) -> CaptureObservation:
        key = (surface, request)
        pending = self.pending.pop(key, None)
        if pending is None:
            raise ValueError("unknown or duplicate completion")
        if pending.target != live_target:
            raise ValueError("stale capture target")
        artifact_hash = digest(data)
        self.blobs.setdefault(artifact_hash, data)
        observation_id = digest(
            (f"capture-observation-v1|{surface}|{request}|{pending.target!r}").encode()
        )
        observation = CaptureObservation(
            observation_id,
            request,
            pending.target,
            pending.source_address,
            artifact_hash,
        )
        self.observations[observation_id] = observation
        return observation


class BrokenCurrentTargetRegistry(CaptureRegistry):
    """Negative control: attaches completion to whatever is current."""

    def complete(self, surface: str, request: int, live_target: CaptureTarget, data: bytes):
        pending = self.pending.pop((surface, request))
        artifact_hash = digest(data)
        return CaptureObservation("broken", request, live_target, pending.source_address, artifact_hash)


class BrokenHashObservationRegistry(CaptureRegistry):
    """Negative control: uses artifact hash as observation identity."""

    def complete(self, surface: str, request: int, live_target: CaptureTarget, data: bytes):
        observed = super().complete(surface, request, live_target, data)
        del self.observations[observed.observation]
        observed = replace(observed, observation=observed.artifact_hash)
        # Equal-byte observations overwrite each other: this is the broken
        # persisted-state behavior the negative control is meant to expose.
        self.observations[observed.observation] = observed
        return observed


@dataclass(frozen=True)
class Envelope:
    observation: str
    target_node: str
    artifact_hash: str


@dataclass(frozen=True)
class GcProposal:
    artifact_hash: str
    reference_revision: int


class CustodyStore:
    def __init__(self, artifact_hash: str, data: bytes) -> None:
        self.blobs = {artifact_hash: data}
        # Each category may have several independent owners of the same blob.
        # Collapsing this to a set of hashes loses reference multiplicity.
        self.refs: dict[str, set[tuple[str, str]]] = {
            "live": set(), "tombstone": set(), "download": set()
        }
        self.revision = 0
        self.tombstones: dict[str, tuple[str, Envelope]] = {}

    def add_ref(self, kind: str, owner: str, artifact_hash: str) -> None:
        self.refs[kind].add((owner, artifact_hash))
        self.revision += 1

    def remove_ref(self, kind: str, owner: str, artifact_hash: str) -> None:
        self.refs[kind].discard((owner, artifact_hash))
        self.revision += 1

    def referenced(self, artifact_hash: str) -> bool:
        return any(
            any(ref_hash == artifact_hash for _, ref_hash in refs)
            for refs in self.refs.values()
        )

    def move_ref(
        self, source_kind: str, target_kind: str, owner: str, artifact_hash: str,
        between=None,
    ) -> None:
        """Move custody state without exposing a zero-reference interval.

        Production needs one serialized actor/transaction boundary around this
        pair.  Add-before-remove is defense in depth for a reader between steps.
        """
        self.add_ref(target_kind, owner, artifact_hash)
        if between is not None:
            between(self)
        self.remove_ref(source_kind, owner, artifact_hash)

    def propose_gc(self, artifact_hash: str) -> GcProposal | None:
        if artifact_hash in self.blobs and not self.referenced(artifact_hash):
            return GcProposal(artifact_hash, self.revision)
        return None

    def apply_gc(self, proposal: GcProposal) -> bool:
        # Revision mismatch is allowed only if a fresh reference check still passes.
        if self.referenced(proposal.artifact_hash):
            return False
        return self.blobs.pop(proposal.artifact_hash, None) is not None

    def delete_to_tombstone(self, node: str, envelope: Envelope) -> None:
        self.move_ref("live", "tombstone", node, envelope.artifact_hash)
        self.tombstones[node] = (node, envelope)

    def restore(self, node: str) -> tuple[str, Envelope]:
        restored = self.tombstones.pop(node)
        envelope = restored[1]
        self.move_ref("tombstone", "live", node, envelope.artifact_hash)
        return restored

    def export(self, node: str, envelope: Envelope, include_attachment: bool) -> dict:
        result = {"node": node, "redacted": []}
        if include_attachment:
            result["capture"] = envelope.observation
        else:
            result["redacted"].append("capture")
        return result


class BrokenGcStore(CustodyStore):
    """Negative control: trusts a stale proposal without rechecking refs."""

    def apply_gc(self, proposal: GcProposal) -> bool:
        return self.blobs.pop(proposal.artifact_hash, None) is not None


class BrokenTransferStore(CustodyStore):
    """Negative control: remove-first transfer exposes a false orphan."""

    def move_ref(
        self, source_kind: str, target_kind: str, owner: str, artifact_hash: str,
        between=None,
    ) -> None:
        self.remove_ref(source_kind, owner, artifact_hash)
        if between is not None:
            between(self)
        self.add_ref(target_kind, owner, artifact_hash)


class R2CaptureTests(unittest.TestCase):
    def setUp(self):
        self.a = CaptureTarget("session-1", "node-a", 7, "surface-4")

    def pending(self, request: int, target: CaptureTarget | None = None):
        return PendingCapture(request, target or self.a, "https://example.test/a")

    def test_exact_target_accepts_once_and_duplicate_refuses(self):
        registry = CaptureRegistry()
        registry.start(self.pending(1))
        observation = registry.complete("surface-4", 1, self.a, b"pixels")
        self.assertEqual(observation.target, self.a)
        with self.assertRaisesRegex(ValueError, "unknown or duplicate"):
            registry.complete("surface-4", 1, self.a, b"pixels")

    def test_navigation_close_and_session_switch_refuse_stale_completion(self):
        variants = [
            replace(self.a, document_generation=8),
            replace(self.a, surface="closed-surface"),
            replace(self.a, session="session-2"),
            replace(self.a, node="node-b"),
        ]
        for request, live in enumerate(variants, 10):
            registry = CaptureRegistry()
            registry.start(self.pending(request))
            with self.assertRaisesRegex(ValueError, "stale capture target"):
                registry.complete("surface-4", request, live, b"pixels")
            self.assertEqual(registry.blobs, {})

    def test_equal_bytes_share_blob_but_keep_distinct_observations(self):
        registry = CaptureRegistry()
        registry.start(self.pending(20))
        registry.start(self.pending(21))
        first = registry.complete("surface-4", 20, self.a, b"same pixels")
        second = registry.complete("surface-4", 21, self.a, b"same pixels")
        self.assertEqual(first.artifact_hash, second.artifact_hash)
        self.assertNotEqual(first.observation, second.observation)
        self.assertEqual(len(registry.blobs), 1)

    def test_request_identity_is_not_reusable_after_success_or_refusal(self):
        succeeded = CaptureRegistry()
        succeeded.start(self.pending(25))
        succeeded.complete("surface-4", 25, self.a, b"pixels")
        with self.assertRaisesRegex(ValueError, "reused request identity"):
            succeeded.start(self.pending(25))
        with self.assertRaisesRegex(ValueError, "unknown or duplicate"):
            succeeded.complete("surface-4", 25, self.a, b"delayed old completion")

        refused = CaptureRegistry()
        refused.start(self.pending(26))
        navigated = replace(self.a, document_generation=8)
        with self.assertRaisesRegex(ValueError, "stale capture target"):
            refused.complete("surface-4", 26, navigated, b"pixels")
        with self.assertRaisesRegex(ValueError, "reused request identity"):
            refused.start(self.pending(26))
        with self.assertRaisesRegex(ValueError, "unknown or duplicate"):
            refused.complete("surface-4", 26, self.a, b"delayed old completion")

    def test_negative_current_target_model_is_detected(self):
        registry = BrokenCurrentTargetRegistry()
        registry.start(self.pending(30))
        navigated = replace(self.a, node="node-b", document_generation=8)
        result = registry.complete("surface-4", 30, navigated, b"pixels")
        self.assertNotEqual(result.target, self.a, "negative control must misattach")

    def test_negative_hash_identity_collapses_observations(self):
        registry = BrokenHashObservationRegistry()
        registry.start(self.pending(40))
        registry.start(self.pending(41))
        first = registry.complete("surface-4", 40, self.a, b"same")
        second = registry.complete("surface-4", 41, self.a, b"same")
        self.assertEqual(first.observation, second.observation, "negative control must collapse")
        self.assertEqual(len(registry.observations), 1, "stored observations must collapse too")


class R2CustodyTests(unittest.TestCase):
    def make(self, store_type=CustodyStore):
        data = b"captured representation"
        artifact_hash = digest(data)
        store = store_type(artifact_hash, data)
        envelope = Envelope("observation-1", "node-a", artifact_hash)
        return store, envelope

    def test_all_reference_retirement_orders_preserve_until_last(self):
        for order in itertools.permutations(("live", "tombstone", "download")):
            store, envelope = self.make()
            for kind in ("live", "tombstone", "download"):
                store.add_ref(kind, f"{kind}-owner", envelope.artifact_hash)
            for index, kind in enumerate(order):
                store.remove_ref(kind, f"{kind}-owner", envelope.artifact_hash)
                proposal = store.propose_gc(envelope.artifact_hash)
                if index < 2:
                    self.assertIsNone(proposal, order)
                    self.assertIn(envelope.artifact_hash, store.blobs)
                else:
                    self.assertIsNotNone(proposal, order)
                    self.assertTrue(store.apply_gc(proposal))
                    self.assertNotIn(envelope.artifact_hash, store.blobs)

    def test_two_live_owners_of_equal_blob_are_independent_references(self):
        store, envelope = self.make()
        store.add_ref("live", "node-a", envelope.artifact_hash)
        store.add_ref("live", "node-b", envelope.artifact_hash)
        store.remove_ref("live", "node-a", envelope.artifact_hash)
        self.assertTrue(store.referenced(envelope.artifact_hash))
        self.assertIsNone(store.propose_gc(envelope.artifact_hash))
        store.remove_ref("live", "node-b", envelope.artifact_hash)
        self.assertIsNotNone(store.propose_gc(envelope.artifact_hash))

    def test_tombstone_restore_preserves_node_and_envelope_identity(self):
        store, envelope = self.make()
        store.add_ref("live", "node-a", envelope.artifact_hash)
        store.delete_to_tombstone("node-a", envelope)
        self.assertTrue(store.referenced(envelope.artifact_hash))
        restored_node, restored_envelope = store.restore("node-a")
        self.assertEqual(restored_node, "node-a")
        self.assertEqual(restored_envelope, envelope)
        self.assertTrue(store.referenced(envelope.artifact_hash))

    def test_apply_rechecks_after_reference_race(self):
        store, envelope = self.make()
        proposal = store.propose_gc(envelope.artifact_hash)
        self.assertIsNotNone(proposal)
        store.add_ref("download", "download-1", envelope.artifact_hash)
        self.assertFalse(store.apply_gc(proposal))
        self.assertIn(envelope.artifact_hash, store.blobs)

    def test_redacted_export_does_not_modify_local_custody(self):
        store, envelope = self.make()
        store.add_ref("live", "node-a", envelope.artifact_hash)
        before = dict(store.blobs)
        exported = store.export("node-a", envelope, include_attachment=False)
        self.assertNotIn("capture", exported)
        self.assertEqual(exported["redacted"], ["capture"])
        self.assertEqual(store.blobs, before)

    def test_negative_stale_proposal_deletes_reintroduced_reference(self):
        store, envelope = self.make(BrokenGcStore)
        proposal = store.propose_gc(envelope.artifact_hash)
        self.assertIsNotNone(proposal)
        store.add_ref("live", "node-a", envelope.artifact_hash)
        self.assertTrue(store.apply_gc(proposal), "negative control must delete")
        self.assertTrue(store.referenced(envelope.artifact_hash))
        self.assertNotIn(envelope.artifact_hash, store.blobs)

    def test_transfer_adds_destination_before_removing_source(self):
        store, envelope = self.make()
        store.add_ref("live", "node-a", envelope.artifact_hash)
        observations = []
        store.move_ref(
            "live", "tombstone", "node-a", envelope.artifact_hash,
            between=lambda current: observations.append(
                (current.referenced(envelope.artifact_hash),
                 current.propose_gc(envelope.artifact_hash))
            ),
        )
        self.assertEqual(observations, [(True, None)])
        self.assertTrue(store.referenced(envelope.artifact_hash))

    def test_negative_remove_first_transfer_exposes_false_orphan(self):
        store, envelope = self.make(BrokenTransferStore)
        store.add_ref("live", "node-a", envelope.artifact_hash)
        proposals = []
        store.move_ref(
            "live", "tombstone", "node-a", envelope.artifact_hash,
            between=lambda current: proposals.append(
                current.propose_gc(envelope.artifact_hash)
            ),
        )
        self.assertIsNotNone(proposals[0], "negative control must expose orphan")
        self.assertTrue(store.referenced(envelope.artifact_hash))


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(__import__(__name__))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    print(json.dumps({
        "probe": "R2-B/C disposable contract model",
        "tests_run": result.testsRun,
        "failures": len(result.failures),
        "errors": len(result.errors),
        "successful": result.wasSuccessful(),
        "hash_substitution": "SHA-256 models equality; production Muniment uses BLAKE3",
    }, sort_keys=True))
    raise SystemExit(0 if result.wasSuccessful() else 1)
