# Profiler-guided backlog

This is a hypothesis list, not a commitment. Re-profile after every accepted patch and let current evidence reorder it.

## Approved cleanup sequence

Work interactively and finish one item before starting the next. Keep every implementation uncommitted until the user reviews the diff and evidence. Use one focused commit per approved item.

1. Gate mimalloc behind a default-off Cargo feature, including the production binary and solver benchmark.
2. A/B test the current head without maintained removable-node indexing (#36). Prefer the old scan if the difference is not measurable or is too small to justify the bidirectional index.
3. A/B test removing the reusable blueprint and part-type-index scratch buffers (#5 and #8). Split them only if the combined result cannot identify a clear decision.
4. A/B test the current head without compact `u32` insertion-option indices (#35). Prefer `usize` if the compact representation has no measurable gain.
5. Draft a private `LayoutNodes` boundary in `layout.rs` for review. If approved, move the existing node storage and its coupled indexes without changing their representation or behavior.
6. Draft a private `ProblemLayouts` boundary in `problem.rs` for review. If approved, move the existing live, detached, sampled, and changed-layout state without changing solver policy.
7. Reconcile the PR report and run the final correctness, behavior, quality, and sustained-throughput gates.

For every A/B test, compare sequentially against the immediate parent with the same target directory, lockfile, allocator feature, compiler, and benchmark input. Reject added complexity when the result is flat. A simpler implementation may replace a faster one only after reporting the measured cost and receiving user approval.

Do not start new optimization experiments during this cleanup sequence.

## Data representation

- Flatten `option_parttype_map: Vec<Vec<InsertionOptionKey>>` only with a dynamic-update design that avoids shifting all later ranges. A naive CSR-style flattening turns each update into O(total options) memmove.
- Consider flattening the remaining per-part-type option buckets only if profiles still justify it. The option store itself is now dense and safely reuses capacity.
- Reuse or flatten short-lived recreate scratch vectors when allocation samples justify it.
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

- Make mimalloc an opt-in Cargo feature so the default build uses Rust's system allocator.
- At `9895def`, the system allocator measured 3.99% slower than mimalloc (`-4.62%..-3.36%`, `p = 0.00`). Users can opt into that gain when their target supports mimalloc.
