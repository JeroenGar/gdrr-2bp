# Profiler-guided backlog

This is a hypothesis list, not a commitment. Re-profile after every accepted patch and let current evidence reorder it.

## Data representation

- Flatten `option_parttype_map: Vec<Vec<InsertionOptionKey>>` only with a dynamic-update design that avoids shifting all later ranges. A naive CSR-style flattening turns each update into O(total options) memmove.
- Consider flattening the remaining per-part-type option buckets only if profiles still justify it. The option store itself is now dense and safely reuses capacity.
- Use `swap_remove` when taking the selected insertion blueprint; both candidate buffers are cleared immediately afterward, so shifting their tails is unnecessary.
- Stream `layouts_to_consider` into initial cache population instead of collecting a temporary vector if the post-blueprint profile still shows that allocation.
- Reuse or flatten short-lived recreate scratch vectors when allocation samples justify it.
- Avoid allocating `get_removable_nodes()` on every ruin selection. First compare a reusable scratch Vec with iterator-based selection.
- Continue removing nested vectors and hash maps from hot state, but benchmark each representation independently.
- Remove redundant getters and setters in a separate cleanup pass. Make plain data fields public unless restricted mutation protects a real invariant.

## Control flow

- Inspect frequently executed nested branches in profiler-confirmed hot functions.
- Reorder logically equivalent conditions so cheap and selective checks run first.
- Look for duplicated expensive predicates and unpredictable branches inside high-count loops.
- Benchmark branch changes in isolation. Do not assume source-level simplification improves generated code.
- Keep the quadratic leftover-valuation fast path while the default configuration uses power `2.0`; re-profile other configured powers before specializing them.

## Profiling practice

- Build with the `profiling` profile so debug symbols remain available.
- Add temporary `#[inline(never)]` annotations progressively around large inlined regions until their internal costs become visible.
- Remove all profiling annotations before acceptance and commit.
- Use fixed-iteration traces for comparable sample totals and a 60-second production run for real throughput and solution quality.

## Allocator end state

- Keep mimalloc while it remains materially faster.
- Re-test system allocator versus mimalloc only after hot-path allocations have fallen substantially.
- If the delta becomes inconsequential, remove mimalloc in its own measured commit to improve portability and compatibility.
