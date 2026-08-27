# Experiment ledger

The PR description is the public accepted-change report. This file also records rejected probes so they are not repeated.

## Current accepted head

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
