# Experiment ledger

The PR description is the public accepted-change report. This file also records rejected probes so they are not repeated.

## Current accepted head

- `25f4452` - store insertion-option node-range endpoints as `u32`, reducing each indexed range entry from 40 to 32 bytes while checking every stored length. Criterion +1.03% versus `64e0e77` (`+0.63%..+1.43%`, `p = 0.00`); exact normalized seeded 50,000-iteration behavior; 60-second result 73,877 iter/s, with a complete 291-sheet solution at 98.464% usage and an incomplete 290-sheet solution at 98.519% usage/99.712% included.
- `84b0ab3` - replace `IOCUpdates`' two heap-allocated vectors with one removed node key and a two-entry stack array containing only new empty nodes. The initial comparison before maintained removable nodes measured +1.23%, but the accepted immediate-parent comparison found no measurable change at +0.04% (`-0.39%..+0.46%`, `p = 0.85`). Exact normalized seeded 50,000-iteration behavior; 60-second result 73,407 iter/s, with a complete 291-sheet solution at 98.464% usage and an incomplete 290-sheet solution at 98.517% usage/99.710% included.
- `b5e6a94` - maintain each layout's removable nodes in a flat key vector with niche-packed reverse positions and O(1) `swap_remove`. Criterion +0.92% versus `c9baf4a` (`+0.16%..+1.61%`, `p = 0.02`); equivalent seeded 50,000-iteration behavior because unordered removal changes the trajectory (parent 298 sheets/96.15138% usage, candidate 299/95.8298%); 60-second result 68,407 iter/s, with a complete 291-sheet solution at 98.4643% usage and an incomplete 290-sheet solution at 98.498% usage/99.690% included.
- `f54e845` - store insertion-option indices as `u32` in the node and part-type reverse lookup buffers. Criterion +0.55% versus `e76d32e` (`+0.15%..+0.91%`, `p = 0.01`); exact normalized seeded 50,000-iteration behavior; 60-second result 66,963 iter/s.
- `0bc4e6b` - copy part-type references directly into the sort buffer and sort them in place. Criterion +1.79% versus `c25a270` (`+1.42%..+2.16%`, `p = 0.00`); exact normalized seeded 50,000-iteration behavior; 60-second result 65,960 iter/s.
- `c25a270` - maintain each layout's used part area at node registration, recursive removal, clone, and restore boundaries. Criterion +5.83% versus `ed5f5a0` (`+5.39%..+6.25%`, `p = 0.00`); exact normalized seeded 50,000-iteration behavior; 60-second result 66,494 iter/s.
- `172d012` - defer rotatable-part fit checks from insertion-cache population until blueprint generation. Criterion +6.88% versus `166aa4e` (`+6.46%..+7.31%`, `p = 0.00`); exact normalized seeded 50,000-iteration behavior; 60-second result 59,991 iter/s.
- `2418100` - calculate layout cost from the maintained empty-node index instead of scanning every SlotMap node. Criterion +6.90% versus `1da65a9` (`+6.55%..+7.25%`, `p = 0.00`); normalized seeded 50,000-iteration output exact, conservatively classified equivalent because `f32` summation order changes; 60-second result 58,021 iter/s.
- `ff0df2b` - reuse the previous solution's `SecondaryMap` and replace only changed layout snapshots. Criterion +2.76% versus `da48351`; exact 50,000-iteration behavior; 60-second result 52,427 iter/s.
- `da48351` - cache excluded part area and maintain it at part registration, removal, and snapshot restore boundaries. Criterion +2.03% versus `65e0153`; exact 50,000-iteration behavior; 60-second result 52,020 iter/s.
- `8701ffa` - sample only the three layouts considered by ruin's low-usage bias instead of allocating and valuing every live layout. Criterion +3.86% versus `84d3cc2`; exact 50,000-iteration behavior; 60-second result 50,210 iter/s.
- `1734ef7` - record each changed layout only once so rejected-solution restore cannot restore the same snapshot repeatedly after several mutations. Criterion +1.68% versus `8e2129e`; exact 50,000-iteration behavior; 60-second result 45,820 iter/s.
- `8e2129e` - stream existing and eligible empty layouts directly into initial insertion-cache population. Criterion +3.64% versus `ef0c65a`; exact 50,000-iteration behavior; 60-second result 45,435 iter/s.
- `ef0c65a` - take selected insertion blueprints with `swap_remove` because both candidate buffers are cleared immediately afterward. Criterion +4.12% versus `3976c34`; exact 50,000-iteration behavior; 60-second result 43,339 iter/s.
- `3976c34` - replace recursive hot insertion-blueprint trees with four compact shapes and at most five stack-backed node descriptors. Criterion +6.05% versus `3579920`; exact 50,000-iteration behavior; 60-second result 41,403 iter/s.
- `3579920` - specialize the default quadratic leftover valuation as direct multiplication. Criterion +3.31% versus `03f56ba`; exact 50,000-iteration behavior; 60-second result 39,236 iter/s.
- `03f56ba` - replace the insertion-option SlotMap with dense Vec storage and retain the cache across recreate phases. Dense storage alone was flat; Vec capacity reuse supplied the measured gain. Criterion +1.08% versus `c766607`; exact 50,000-iteration behavior; 60-second result 38,510 iter/s.
- `c766607` - skip the oversized prefix of area-sorted part types once per empty node. Criterion +1.2% versus `7cb9743`; exact 50,000-iteration behavior; 60-second result 38,255 iter/s.
- `7cb9743` - reuse the removable-node selection buffer across ruin steps. Criterion +3.7% versus `4ce5b00`; exact 50,000-iteration behavior; 60-second result 35,574 iter/s.
- `4ce5b00` - cache each insertion option's part-type vector position. Criterion +7.3% versus `4d726cf`; exact 50,000-iteration behavior; 60-second result 35,156 iter/s.
- `4d726cf` - reuse live layout allocations during rejected-solution restore. Criterion +3.0% versus `ebc175b`; exact behavior; 33,830 iter/s.
- `ebc175b` - replace per-node child vectors with intrusive typed SlotMap links. Criterion +11.3% versus `d052159`; exact behavior; 32,845 iter/s. Fixed 500,000-worker time 19.7s to 17.6s, restore 2.52s to 1.77s, Node clone 363ms to 88ms.
- Earlier accepted changes and their measurements are documented in PR #6.

## Rejected experiments

### Specialize unrestricted-rotation option generation after `14168d1`

- Detected instances where every part permits both rotations, selected that mode once per empty node, and monomorphized the inner option-generation loop without the per-part fixed-rotation match.
- Full-solver Criterion measured a 5.70% throughput regression, with the entire confidence interval below zero (`-6.37%..-5.10%`).
- Rejected immediately. The extra helper boundary and larger generated loop cost much more than the already predictable `Option<Rotation>` branch.

### Sort empty-node keys once per recreate after `4a8f105`

- Kept each layout's empty-node key vector unordered during ruin and recreate mutations with `push` and `swap_remove`, then sorted only dirty layouts once before initial insertion-cache population.
- Full-solver Criterion measured a 5.05% throughput regression, with the entire confidence interval below zero (`-5.63%..-4.44%`).
- Rejected immediately. Even one unstable sort per recreate costs more than the existing sorted insertion and area-bounded removal. This is distinct from the earlier failed probe that sorted after every public layout mutation; do not try a third batching boundary without a different ordering algorithm.

### Cache the active part-area cutoff after `bc04200`

- Kept the current area-sorted part type's area in one scalar while advancing the monotonic cutoff during initial cache population, avoiding a repeated `PartType::area` load when the cutoff did not move.
- An initial ten-sample comparison was inconclusive at +0.43%. The isolated 20-sample comparison found no gain: -0.22% throughput with a `-0.66%..+0.21%` confidence interval and `p = 0.33`.
- Reverted because three extra lines and explicit loop state did not improve the full solver. The temporary no-inline profile over-attributed surrounding optimized work to the source line, or the production build already retained the value effectively.

### Sort the active part-type vector in place after `90f945a`

- Let initial cache population area-sort the recreate loop's existing active part-type vector instead of allocating and sorting a second copy.
- Full-solver Criterion measured a 1.81% throughput regression, with the entire confidence interval below zero (`-2.48%..-1.19%`).
- Rejected immediately. Removing one allocation did not repay carrying area order into later selection and incremental cache updates; the current separate sorted copy preserves the better downstream order.

### Normalize rotatable insertion dimensions after `415c21f`

- Replaced the default-or-rotated fit checks used during initial cache population with one comparison of each rectangle's shorter and longer sides.
- Full-solver Criterion measured a 1.93% throughput regression, with the entire confidence interval below zero (`-2.53%..-1.32%`).
- Rejected immediately. The existing check often accepts the default orientation after two comparisons; normalizing both rectangles always pays for the extra minimum and maximum operations.

### Precomputed part-type area order after `5fcf708`

- Stored one stable descending-area part-type index order in immutable `Instance`, filtered it for active quantities, and passed the already sorted references into initial cache population.
- Full-solver Criterion measured a 2.21% throughput regression, with the entire confidence interval below zero (`-2.74%..-1.66%`).
- Rejected immediately. Avoiding the small per-recreate reference sort did not repay the extra indexed traversal through `Instance`; the existing direct-reference sort has better locality.

### Force-inline scalar size accessors after `e30de3b`

- Added `#[inline(always)]` to `Size::width`, `Size::height`, and `Size::area` after the symbolized profile attributed 4.7% of worker leaf samples to the accessors.
- Full-solver Criterion found no change: +0.09% median throughput with a `-0.44%..+0.68%` confidence interval and `p = 0.77`.
- Reverted because the leaf symbols represented surrounding optimized work rather than meaningful call overhead; forced inlining adds annotation noise without improving the solver.

### Direct solver cost comparator after `f54e845`

- Removed `GDRR`'s stored function pointer and called the fixed solver cost comparator directly, while preserving the public configurable comparator used by solution collectors.
- Full-solver Criterion found no gain: -0.32% median throughput with a `-1.02%..+0.38%` confidence interval and `p = 0.37`.
- Reverted because the net code deletion did not improve the full solver; compiler optimization already removes enough of the indirection or the remaining call cost is immaterial.

### Single-pass sampled layout resolution after `f54e845`

- Drew the same three random layout positions, sorted them, and resolved their `SlotMap` keys with one forward traversal instead of three independent `keys().nth(position)` traversals.
- Before compact insertion-option indices landed, the probe measured +1.29% and an isolated 20-sample comparison measured +0.76% throughput (`+0.35%..+1.19%`, `p = 0.00`). Rebased onto the actual immediate parent, full-solver Criterion found no change: +0.32% with a `-0.16%..+0.75%` confidence interval and `p = 0.19`.
- Reverted because the gain did not survive composition with the preceding accepted change, and 25 lines of position-resolution bookkeeping are not justified without a measurable improvement.

### Compact insertion-cache positions after `00f3b6a`

- Stored each cached option's part-type and node reverse positions as `u32`, shrinking `CachedInsertionOption` from 56 to 48 bytes.
- Full-solver Criterion found no gain: -0.54% median throughput with a `-1.05%..-0.01%` confidence interval and `p = 0.08`.
- Reverted because the smaller cache entry did not improve the full solver and adds integer conversion and a representable-size invariant.

### Circular sibling-tail links after `ce15909`

- Removed each node's last-child key and stored the tail key in the first child's previous-sibling field, shrinking `Node` from 72 to 64 bytes and each SlotMap slot from 80 to 72 bytes.
- An initial ten-sample probe measured +1.41%, but an isolated 20-sample comparison found no change: +0.06% median throughput with a `-0.70%..+0.74%` confidence interval and `p = 0.86`.
- Reverted because the stronger result did not reproduce the gain, and overloading the first child's previous-sibling link makes list mutation harder to reason about.

### Singly linked layout nodes after `4e3f903`

- Removed each node's previous-sibling key and found the predecessor by scanning the parent's children during removal.
- Full-solver Criterion found no gain: -0.33% median throughput with a `-0.85%..+0.20%` confidence interval and `p = 0.28`.
- The compiler did shrink `Node` from 72 to 64 bytes and each SlotMap slot from 80 to 72 bytes, but the extra removal scan canceled that representation benefit.

### Dense node SlotMap after `234f9b5`

- Replaced each layout's `SlotMap<NodeKey, Node>` with `DenseSlotMap<NodeKey, Node>` while keeping the same typed stable keys.
- Full-solver Criterion measured a 10.07% throughput regression, with the entire confidence interval below zero (`-10.53%..-9.57%`).
- Rejected immediately. Dense iteration and cloning did not compensate for the extra indirection on frequent keyed node access.

### Binary part-type area cutoff after `9d0013d`

- Replaced the monotonic linear scan over area-sorted part types with `partition_point` on the remaining suffix.
- Full-solver Criterion measured a 0.88% throughput regression, with the entire confidence interval below zero (`-1.57%..-0.18%`).
- Rejected immediately. The solver's short active slices favor the existing linear merge.

### Compact part-type storage after `84fd040`

- Removed the duplicate width and height fields from `PartType` and read them from its existing `Size` value instead.
- Full-solver Criterion found no change: +0.06% median throughput with a `-0.40%..+0.49%` confidence interval and `p = 0.79`.
- Reverted because the smaller representation did not improve the full solver.

### Remove redundant insertion-option rotation after `172d012`

- Removed `InsertionOption::rotation` after cache-created options began storing the same value as `PartType::fixed_rotation`.
- An initial ten-sample probe measured +0.74%, but an isolated 20-sample comparison found no change: +0.17% median throughput with a `-0.22%..+0.51%` confidence interval and `p = 0.39`.
- Reverted because `InsertionOption::new` is public and its rotation argument can intentionally restrict an otherwise rotatable part. A flat result does not justify breaking that API or removing the behavior it can represent.

### Cache part-type areas during insertion-cache population after `572db05`

- Cached each part type's area beside its reference while sorting, then reused it when skipping oversized prefixes for empty nodes.
- An initial ten-sample probe measured +2.42%, but an isolated 20-sample comparison found no change: +0.91% median throughput with a `-0.74%..+2.15%` confidence interval and `p = 0.20`.
- Reverted because the stronger result did not reproduce the gain; the extra tuple field is not justified.

### Constant-time selected part-type removal after `572db05`

- Returned the selected part-type vector index and removed exhausted or infeasible entries with `swap_remove` instead of scanning the vector with `retain`.
- An initial ten-sample probe measured +2.94%, but an isolated 20-sample comparison found no change: +0.88% median throughput with a `-0.67%..+1.90%` confidence interval and `p = 0.23`.
- Reverted because the stronger result did not reproduce the gain, while unordered removal also changes the seeded solver trajectory.

### Batch empty-node index maintenance after `3d5d6f8`

- Replaced sorted insertion and shifting removal in each internal node mutation with `push` and `swap_remove`, then restored descending area order once at the public layout-mutation boundary.
- Full-solver Criterion measured a 1.75% throughput regression, with the entire confidence interval below zero (`-2.76%..-0.85%`).
- Rejected immediately. The final stable sort costs more than the avoided shifts on the solver's small empty-node lists.

### Unstable part-type sorting after `03f56ba`

- Replaced the recreate cache's area sort with `sorted_unstable_by`.
- Full-solver Criterion was 2.54% slower than the immediate parent, with the entire confidence interval below zero throughput (`-4.06%..-0.94%`).
- Reverted immediately. The small slices favor the current stable sort implementation despite its visible profiler samples.

### Unstable blueprint ranking after `03f56ba`

- Replaced the existing-layout blueprint `sort_by` with `sort_unstable_by`.
- Full-solver Criterion found no change: +0.63% median throughput with a `-0.96%..+2.36%` confidence interval.
- Reverted because it changes equal-cost tie ordering without a measurable gain.

### Callback-based removed-part reporting after `03f56ba`

- Replaced the fresh removed-part ID vector with an allocation-free callback from `Layout` to `Problem`.
- Full-solver Criterion found no change: -0.32% median throughput with a `-1.99%..+1.24%` confidence interval.
- Reverted rather than add callback plumbing without a measurable gain.

### Reuse recreate scratch vectors after `9fae23e`

- Retained the part-type, blueprint, and selection-index vectors across ruin/recreate iterations.
- The full-solver fast gate was about 5.1% slower, with a throughput confidence interval of `-10.10%..-1.20%`.
- Reverted immediately. As with `SlotMap::clear`, retaining high-water storage hurt more than avoiding these allocations helped.

### Reuse biased-sampler layout storage after `274f1ba`

- Changed `BiasedSampler` to borrow a retained layout buffer instead of owning a fresh vector for each ruin selection.
- Full-solver Criterion found no gain: -1.31% median throughput with a `-2.81%..+0.13%` confidence interval.
- Reverted because the sampled code is too small to justify changing the sampler's ownership API.

### Reuse part-type sorting storage after `3976c34`

- Retained the temporary vector used to area-sort eligible part types inside `InsertionOptionCache`.
- The probe improved Criterion throughput by 1.88% before insertion blueprints were flattened, but only 0.39% afterward with a `-1.39%..+2.13%` confidence interval.
- Reverted because the gain did not survive against the new immediate parent.

### Compare insertion dimensions directly after `8e2129e`

- Replaced `Size` selection in `Node::insertion_possible` with direct width and height comparisons.
- Full-solver Criterion measured a 1.69% throughput regression, with the entire confidence interval below zero (`-2.26%..-1.12%`).
- Reverted because avoiding the small `Size` selection made the full solver slower.

### Height-first insertion fit check after `8e2129e`

- Reversed the equivalent width and height short-circuit checks in `Node::insertion_possible` to reject tall options first.
- An initial ten-sample probe measured +0.95%, but an isolated 20-sample comparison found no change: +0.16% median throughput with a `-0.18%..+0.52%` confidence interval and `p = 0.38`.
- Reverted because the stronger result did not reproduce the gain.

### Reuse quantity-vector allocations on restore after `8e2129e`

- Used `Vec::clone_from` for part-type and sheet-type quantity restoration.
- Full-solver Criterion measured a 1.88% throughput regression, with the entire confidence interval below zero (`-2.41%..-1.39%`).
- Reverted because retaining those vector allocations was slower than fresh clones.

### Reuse InsertionOptionCache with `SlotMap::clear`

- Tried again immediately after `4ce5b00`.
- Kept the cache across recreate cycles and cleared its SlotMap and vectors in place.
- Criterion warm-up estimated roughly 45 seconds for ten samples versus roughly 20 seconds for the parent, about a 2x regression. Collection was stopped early and the patch reverted.
- This repeats an earlier failed SlotMap-clear/reuse attempt. Do not try it a third time without changing the representation or explaining why SlotMap high-water/free-list behavior no longer applies.

### Restore allocation reuse, first comparison

- The first parent worktree regenerated an ignored `Cargo.lock` and resolved different dependency versions. Its flat result was invalid.
- Repeating both sides with the same dependency set showed a real +3.0% gain and produced accepted commit `4d726cf`.
- Lesson: never trust a detached-worktree benchmark until dependency identity is verified.

## Result interpretation

- Intrusive child links were counterintuitively faster despite linked traversal because removing per-node Vec allocation and clone/drop work dominated traversal locality.
- Direct cache positions gained much more than the linear-search leaf percentage suggested, likely because they also reduced surrounding mutation and cache costs.
- Allocation reuse is not automatically faster. Retained capacity and free-list traversal can dominate saved allocator calls.
