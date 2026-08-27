# Core readability audit

This audit covers solver and domain types, not `util`, debug assertions, or presentation helpers. Apply one narrow commit at a time; benchmark any hot-path change and revert it when throughput regresses.

## Keep fields private

These types contain derived state or cross-field invariants. Their accessors are intentional boundaries.

- `Size`: cached area must match width and height.
- `PartType`: default and rotated sizes must agree with its dimensions.
- `Node` / `LayoutNodes`: intrusive links, removable positions, used area, and node indexes must stay synchronized.
- `Layout`: node state and cached cost must stay synchronized.
- `Instance`: dense IDs and cached totals depend on the part and sheet collections.
- `Problem` / `ProblemLayouts`: quantities, excluded area, live keys, snapshots, and changed membership move together.
- `InsertionBlueprint`: replacement shape and cached cost are derived together.
- `IOCUpdates`: one removed node and at most two new empty nodes describe one mutation.
- `InsertionOptionCache` / `CachedInsertionOption`: dense storage and reverse indexes move together.
- `ProblemSolution` / `SendableSolution`: layouts, quantities, cost, and usage form one snapshot.
- `LocalSolCollector` / `GlobalSolCollector`: best solutions, transfer flags, and material limits form coordination state.

## Plain-record candidates

Test separately; public-field conversion is appropriate only where the constructor currently accepts every representable state.

- Accepted: `SheetType` is now a public-field record because its input values are independent and it has no cached derived fields.
- Accepted: `NodeBlueprint` and `SendableLayout` are public-field transport records; their trivial accessors, unused `add_child`, and panicking `convert_to_layout` API are gone.

## Accessor experiments

- Rejected: public `InsertionOption` fields regressed full-solver throughput by 1.66%; keep its private hot representation.
- Accepted: `PartType::fixed_rotation` returns its small `Copy` option by value with no measurable throughput change.
- Rejected: returning hot `NodeKey` and `LayoutIndex` values by value regressed throughput by 1.02%; keep those accessors borrowed.
- Remove repository-unused accessors only with an explicit public-API decision; they may still be external API.
- Rename Java-style `get_*` methods only as a separate public-API cleanup.
- Accepted: core `get_*` methods now use Rust noun names, manual quantity matches use `Option::map`, and repository-unused core accessors are removed.

## Core control-flow cleanup

- Prefer `if let`, `let ... else`, `Option` combinators, and tail expressions when they remove branching ceremony in solver/domain code.
- Keep mutation-heavy loops imperative; do not introduce iterator chains merely for style.
- Benchmark changes in `gdrr`, `problem`, `layout`, insertion generation, and cache code. Cold snapshot/collector cleanups need correctness checks but no sustained performance run.
- Accepted: solution creation uses a lazy cached-cost fallback, avoiding a discarded full-layout scan and improving adjacent Criterion throughput by 2.84%.
