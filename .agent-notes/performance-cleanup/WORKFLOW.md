# Performance cleanup workflow

Read this file and `EXPERIMENTS.md` before starting or repeating an experiment.

## Discovery lane

1. Profile the latest accepted commit, not an unverified stack of changes.
2. Use the full solver with a fixed seed and representative large instance.
3. Let the profiler choose the target. Distinguish inclusive time from leaf cost and allocator or memmove descendants.
4. When inlining hides a large block, add `#[inline(never)]` to only the suspected functions, profile again, then remove every annotation.
5. Form one narrow hypothesis and implement the smallest isolated experiment.
6. Run `cargo check --all-targets`.
7. Run the short, full-solver Criterion comparison sequentially against the immediate parent.
8. Abort early and revert when it is clearly slower. Do not spend time on correctness, sustained runs, or PR work for a failed performance gate.

## Acceptance lane

Delegate a promising candidate to a sub-agent in a separate worktree while the primary agent profiles and starts the next experiment. Snapshot the candidate on a temporary branch, return the primary checkout to the accepted head, and never make the discovery lane wait on acceptance. The acceptance owner must:

1. Audit semantics, invariants, and the complete diff.
2. Compare seeded 50,000-iteration behavior with the immediate parent. Exact output is preferred; equivalent solver behavior is acceptable and must be explained.
3. Run three debug iterations on the large fixture so debug assertions exercise structural and cache invariants. Do not use the 100-iteration verification config here: rebuilding the insertion cache after every mutation makes that take several minutes.
4. Run the 60-second production solver and record iterations per second plus complete and incomplete solution quality.
5. Run `cargo check --all-targets`, `cargo test --lib`, and `git diff --check`.
6. Commit only the accepted experiment, push it, update the PR table and numbered section, and verify the exact pushed revision in CI.

The primary agent and acceptance owner must use separate worktrees. Coordinate before integrating an accepted commit if the discovery lane has another candidate in flight. Performance measurements may run concurrently when the machine is idle, but record that they overlapped; prefer isolated runs for final numbers.

## Benchmark hygiene

- Criterion measures the whole solver for 50,000 iterations with mimalloc, matching the production binary.
- Compare immediate parent and candidate with the same compiler, dependencies, allocator, benchmark input, and target directory.
- `Cargo.lock` is ignored in this repository. A detached worktree may resolve different dependency versions. Prefer measuring parent and candidate sequentially in the same worktree, or explicitly make their lockfiles identical.
- Stop a Criterion collection early when warm-up already shows a severe regression.
- Treat sample counts as evidence about where to investigate, not as throughput claims.
- Record Criterion delta versus parent, 60-second iterations per second, solution quality, behavior status, and commit hash in PR #6.
- Keep every direct `gdrr_main` run entirely outside `~/Documents`: put `CARGO_TARGET_DIR`, the executable itself, input, config, optional output, trace files, and command working directory under `/tmp`. Launching `target/.../gdrr_main` from this repository can trigger macOS Documents-folder permission prompts even when its arguments point to `/tmp`. Output paths are optional and should stay omitted for profiling and verification.

## Design constraints

- Keep hot state flat and Vec-backed where practical.
- Avoid `Vec<Vec<_>>`, hashing, per-element heap ownership, and pointer-heavy trees in the hot path.
- `Rc` is acceptable for large immutable snapshots such as `Instance` or shared `Layout` snapshots.
- Prefer public fields for plain data. Restrict fields only when mutation can break a real invariant, such as intrusive node links.
- Add focused debug assertions for implicit cache and structural invariants. Avoid permanent fine-grained test sprawl.
- Preserve solver policy and conceptual behavior.
