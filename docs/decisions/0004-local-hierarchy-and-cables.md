# 0004 — Local hierarchy, terminal exposure, and cable grouping

Status: accepted for the first hierarchy vertical slice.

## Scope and identity

Implement project-local functional blocks with dedicated occurrence IDs, nested parents, and device ownership. A `SubsystemId` always means a placed block, never a reusable definition or revision. Full shared templates, immutable definition content, instantiation/remapping, and project update adoption remain separate work; do not imply that local blocks already provide those capabilities.

Keep physical location and drawing placement independent of functional membership. Reparenting a block or moving a device between scopes preserves its IDs and actual electrical references.

## Interface semantics

An interface port exposes an existing terminal owned by a contained device. It is not another physical terminal and does not split or duplicate a conductor. An ancestor can expose a deeper terminal only when every intervening child boundary already exposes it. Names are unique per block, not globally; several named exposures of the same terminal are allowed.

A completed conductor crossing scopes requires explicit terminal exposure at each boundary it exits, up to the lowest common containing block. Wires between sibling blocks need each child's interface but do not require the common parent's external interface. Unassigned ends remain valid incomplete engineering. Actual endpoints remain terminal IDs, so overview and detailed views describe the same objects.

This is an initial direct-terminal interface model. Whole connector interfaces, terminal bridges, splices, and richer signal/net semantics need their own requirements. It does not claim to resolve every electrical net or automatically infer internal continuity between different terminal numbers.

## Cables

A cable groups existing conductor IDs and names ordered overview endpoints as devices or subsystems. Conductor A/B ends correspond to cable A/B. A core can belong to only one cable. Assigned core terminals must belong to the designated device or be exposed by the designated block; incompatible detail edits are rejected before mutation. Empty cables and spare cores are valid drafts.

Removing a cable ungroups its cores without deleting them. A grouped core cannot be deleted until the assembly is removed or replaced deliberately. Libraries, conductor attributes, shields, and pairing remain later capabilities.

## Validation and performance

Use private ordered indexes for terminal ownership, terminal exposures, and cable membership. Ordinary termination edits check the affected ancestry/interfaces and their cable endpoints. Reparenting, device-scope edits, and port removal provisionally validate the whole model; failure restores the previous state without exposing intermediate edits. Profile before adding dependency-invalidation machinery.

Traversal is iterative and identity ordered. Adopt a provisional 256-level guard against excessive parent-chain work, with tests at and beyond the boundary. This guard is not a claim of optimal capacity or a substitute for total object/memory/expansion budgets. The local scale probe measures unrelated block growth and depths 16/128/256 while retaining complete validation and undo checks.

## Persistence and history

Write schema v3 with explicit subsystem, device-owner, interface, and cable data. Parse subsystems in deterministic parent-first order regardless of file array order, then devices, child-first exposures, conductors, and cables. Preserve canonical output ordering independently of dependency order. V2 upgrades losslessly into root-level objects; v1 still requires explicit project-ID migration. Unknown fields in older formats must not become accepted simply because a newer schema recognizes them.

Add all structural edits to the existing transaction/inverse mechanism. The controller/conveyor fixture checks the same terminal and conductor identities through construction, detail changes, failed removals, undo/redo, and guarded save/reopen.
