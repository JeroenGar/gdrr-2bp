---
name: writing-rust-code
description: Use when writing, modifying, debugging, testing, refactoring, optimizing, or reviewing Rust in Holon, including .rs files, Cargo crates, libraries, binaries, APIs, domain models and types, control flow, parsers, adapters, ETL and data processing, services, stateful data structures, numerical code, optimization and search algorithms, and performance-sensitive kernels.
---

# Rust Engineering

Follow these project-wide Rust engineering conventions: make correctness assumptions and invariants explicit, fail loudly on broken internal state, preserve readable domain-oriented dataflow, keep abstractions narrow, and optimize measured bottlenecks without compromising robustness. Apply them to APIs, ordinary application code, ETL pipelines, and performance-critical algorithms alike.

When engineering goals compete, preserve correctness and explicit semantics first. Prefer readable, maintainable dataflow by default; let measured performance override style locally only when correctness remains independently verifiable.

## Work From Correctness To Evidence

1. Trace the data, control, and mutation flow end to end.
2. Define the observable result, correctness invariants, and known input or resource limits.
3. Choose the simplest implementation that meets those limits. Treat state and indexes inherent to the chosen algorithm as baseline; do not design for hypothetical scale.
4. Verify small fixtures and edge cases. When performance matters, measure a representative optimized workload.
5. Optimize only a known or measured bottleneck. Prefer the existing ownership boundary; add an abstraction only when it protects an invariant, supports real reuse or variation, or isolates a replaceable kernel.
6. Recheck the same correctness boundary and workload. Keep additional state and complexity only when the benefit justifies them.

## Express Rust Semantics Directly

Choose control flow by meaning:

- Use `let ... else` when a value is required for the successful path and absence or failure exits the current scope.
- Use `if let` when one pattern matters and every other case intentionally has the same behavior.
- Use `match` when cases have distinct behavior or exhaustiveness should make evolution compile-visible.

Introduce a newtype when mixing values would be a realistic correctness bug or the wrapper enforces an invariant; do not wrap strings or numbers merely to relabel them. Replace a boolean parameter with an enum when named modes make the call site clearer or additional states are plausible; keep obvious local predicates as booleans.

## Separate Representations By Responsibility

Use distinct types when responsibilities require different data, invariants, or operations. Do not create a second type only to label the same value as proposed, packed, cached, or committed; let the owning container provide that context.

- Keep external or deserialized records at the trust boundary.
- Validate and normalize boundary data as it enters the system. Materialize bounded inputs into immutable, precomputed internal data; process large or streaming inputs incrementally instead of collecting the whole dataset.
- Keep mutable algorithm or pipeline state in the object that owns its invariants and caches.
- Represent checkpoints and results as immutable values containing only what is needed to compare, restore, or report.

Do not carry strings, nested boundary structures, or repeated metadata through hot code when compact internal values are sufficient.

Prefer ownership transfer for phase-level transformations: accept `T` and return the transformed `T` or next-phase type when the caller relinquishes the original; use `&mut T` for ordinary in-place updates within the caller's ownership context. Treat cloning a large aggregate as an explicit cost even outside hot loops—move it when the caller is done, and make callers that need both versions opt into the clone. Share large immutable structures when independent states or workers need the same data, and clone mutable state only when independent ownership or rollback genuinely requires a separate copy.

## Engineer Algorithms And Hot Paths

Apply the following layout and performance guidance when data volume, algorithmic complexity, or measured runtime makes representation choices material.

### Choose Data Layout From Operations

Prefer contiguous storage and direct indexing when identity is stable and the key space is dense. Use typed indices when confusing two index domains would be a correctness bug.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StateIdx(usize);

fn value(values: &[f64], index: StateIdx) -> f64 {
    values[index.0]
}
```

Choose identity separately from physical storage. When insertions and removals must not invalidate references, use stable handles. If hot numeric work also needs dense rows, map stable handles to dense indices and keep the hot values in vectors.

Exploit known structure behind a small API:

- Flatten dense multidimensional data when it improves locality.
- Store only one triangle of symmetric pair data.
- Use arrays for genuinely fixed arity.
- Use unordered removal when order has no meaning.

Keep index arithmetic in one place and assert its bounds, symmetry, or round trips in debug builds.

Choose sorted vectors, maps, sets, or specialized structures from the measured mix of lookup, traversal, insertion, and removal. Prefer the standard library until a different representation proves useful.

Put representation invariants on the type that owns them. Keep invariant-bearing fields private, validate them in the smallest constructor, and expose only read-only views and mutations that preserve the invariant. Call that constructor from parsers instead of duplicating its checks; express simple invariants directly, such as `windows(2)` for sorted order.

Keep frequently accessed fields compact. Split hot and cold data only when measurement shows that carrying the cold fields through the kernel matters.

### Shape Data And Hot Paths

Treat memory traffic, working-set size, and access predictability as algorithm costs. Prefer compact contiguous storage, traversal in storage order, and batching that reuses nearby data. Avoid pointer-rich structures in hot paths unless their operations justify the locality cost.

Recompute cheap derived values from nearby authoritative data instead of adding storage, indirection, and invalidation logic. Precompute only when the work is expensive or reused enough to justify it; compare both approaches on an optimized representative workload.

Keep tight loops free of allocation, cloning, formatting, and temporary hash tables. Allocate reusable buffers outside the loop and derive each result from an immutable baseline rather than cumulatively modifying prior results.

Order work from cheap and likely to reject toward expensive and exact. Stop when the caller's question is answered; distinguish "find any" from "collect all" when early return avoids work.

Add parallelism only after measuring a CPU bottleneck with independent work. Give workers independent mutable state, share immutable input, and combine results deterministically when reproducibility matters.

## Keep Boundaries Small

Make each phase expose the smallest meaningful input and output contract. Keep orchestration focused on the sequence of phases, not their internal decisions.

Keep one authoritative implementation of each domain behavior. Reuse it for validation, diagnostics, simulation, and reporting—using isolated state or rollback where necessary—instead of re-encoding its decisions elsewhere.

Keep temporary concepts, one-phase helpers, and intermediate representations local to the phase that needs them. Promote an abstraction only for reuse across meaningful boundaries or when it protects an invariant.

Split a file when unrelated responsibilities make the algorithm hard to follow, not because it crossed an arbitrary line count.

Let each phase own concise diagnostics. Report summaries at phase boundaries and keep per-element logging out of hot loops.

## Keep Logic Readable

Make structure, domain names, and semantic checkpoints expose the algorithm's model, assumptions, phases, state transitions, and policy choices. A reviewer should not need to simulate individual expressions to understand the flow.

Place exposed types and their `impl` blocks before private supporting types. Define private structs and enums below the exposed implementation they support instead of collecting every type at the top of the file.

Order inherent `impl` methods by lifecycle:

1. Constructors and factory functions.
2. Exposed mutating methods.
3. Private mutating helpers.
4. Exposed read-only methods.
5. Private read-only helpers.

Treat visibility at the type's intended boundary, including `pub(crate)` and `pub(super)`, as exposed. Within each group, follow lifecycle rather than alphabetical order. Keep paired operations together and use their conventional order, such as add before remove.

Use a named semantic checkpoint when a non-trivial intermediate value represents a meaningful algorithm stage. Use a block initializer to keep local setup, branching, or scratch state behind that name. Keep the checkpoint near its consumer and lazy unless materialization is required. Compute and name a non-trivial query before matching on its outcome, so the match expresses the decision rather than the query mechanics. Do not name obvious expressions.

```rust
let best_feasible_candidate = {
    let candidates = generate_candidates(state);
    candidates
        .filter(|candidate| constraints.allow(candidate))
        .min_by_key(|candidate| candidate.cost())
};

match best_feasible_candidate {
    Some(candidate) => apply(candidate),
    None => stop_search(),
}
```

When a fallible phase is sequential, make its successful path read top to bottom, propagate intermediate failures uniformly, and handle the final outcome separately.

Use comments only for contracts, assumptions, important side effects, early termination, or non-obvious policy. Prefer names and structure over commentary; remove stale comments.

## Express Dataflow Clearly

Prefer iterator pipelines for transformations, queries, reductions, and short-circuiting when they read directly from input to result. Use semantic combinators, including `itertools` when it gives the domain operation a clearer name. Validate equal lengths before zipping when truncation would hide malformed input.

Break long pipelines at named semantic checkpoints when their stages would otherwise be hidden.

Keep closures short and expression-oriented. Pass a function or method directly when a closure only forwards its arguments.

Return `impl Iterator` from reusable traversals when callers benefit from composition or short-circuiting. Materialize only for sorting, shuffling, indexing, reuse, ownership transfer, or a snapshot before mutation; do not collect merely to resume iteration.

Use a normal loop when stateful control flow is the algorithmic story: mutation, rollback, borrow coordination, or several evolving accumulators.

## Separate Policy From Mechanism

Apply this separation when the change selects, ranks, evaluates, or repeatedly applies candidate operations.

Mechanism generates and applies possible operations while preserving state invariants. Policy decides which operations are eligible and preferred.

Keep core state and mutation concrete. Isolate a policy only when it genuinely varies, such as eligibility, ordering, evaluation, sampling, termination, or progress reporting. Use a direct closure or named function for an obvious stable policy; introduce a trait when real interchangeable implementations exist.

Use generics or `impl Trait` for policies called from hot code. Use dynamic dispatch only when runtime heterogeneity is required and its cost is irrelevant or measured.

Separate selection into three responsibilities:

- Eligibility decides whether an operation is allowed.
- Ordering chooses among allowed operations.
- Application performs the chosen operation and restores invariants.

```rust
let selected_move = candidate_moves(state)
    .filter(|candidate| is_eligible(state, candidate))
    .min_by_key(|candidate| ordering_key(state, candidate));

if let Some(candidate) = selected_move {
    state.apply(candidate);
}
```

Here, `candidate_moves` is the exploration mechanism, `is_eligible` and `ordering_key` express policy, and `apply` owns mutation and invariant restoration.

Keep non-trivial or heuristic ordering in a named key, score, or comparison function. Model semantic outcomes such as valid, invalid, complete, or partial explicitly and define their precedence before comparing numeric quality. Do not hide domain precedence in sentinel values or unexplained weighted scalars.

Represent a proposed operation faithfully. Preserve every effect needed for validation and application even when ordering reduces it to a smaller key. Use deterministic tie-breakers in tests, benchmarks, and seeded runs; treat randomized tie-breaking as an explicit policy.

Separate exploration from evaluation. Let the explorer own traversal and refinement, and the evaluator own the expensive domain calculation. Pass the current bound into the evaluator when it can stop once the result is known to be uncompetitive.

Keep reporting and cancellation outside the algorithmic state. These hooks may observe or interrupt the work, but should not own the state being optimized.

## Control Mutation

Expose immutable inspection freely. Route mutation through narrow methods that leave the state consistent before returning.

Keep mutable borrows short: inspect first, decide, then mutate. Prefer validating an operation before mutation when that is cheap and exact.

For trial, rollback, branch, or independent-worker updates, choose a restoration strategy according to frequency and state cost:

- Use apply/undo for tight trial operations only when undo restores every changed field bit-exactly.
- Record overwritten values or use a compact snapshot for floating-point, lossy, cache-heavy, phase, branch, or independent-worker updates.
- Use differential restoration when rebuilding expensive unchanged structures would dominate.

Treat caches, counters, and indexes as derived state. Keep one authoritative source of truth and update derived state at the same mutation boundary.

## Isolate Verification From Implementation Logic

Keep implementation files focused on algorithmic and domain logic. For each non-trivial directory-backed module, put tests of exposed behavior in `tests.rs` and complex invariant checks in `assertions.rs`, declared from the module root:

```rust
mod assertions;

#[cfg(test)]
mod tests;
```

In `tests.rs`, exercise structs and functions through the same exposed API used by real callers, covering observable success, errors, rollback, and resulting state. Test private helpers implicitly through this flow; do not test them directly or widen visibility for tests.

Use assertions for internal invariants. Keep public preconditions and cheap local assertions beside the methods they guard, preferably as a single line. Move multi-step invariant recomputation to `assertions.rs`; invoke it with `debug_assert!` when it would add production overhead.

Recompute expected state independently from authoritative objects only. Never use cached or derived values as assertion-oracle inputs, even indirectly; two stale values can agree and hide drift. Do not promote broken internal invariants to public errors merely to make them testable. Ensure the test profile enables debug assertions; when relevant, exercise them on a representative optimized workload and benchmark without them unless they are part of production.

## Make Evolution Compile-Visible

When a method consumes most fields of a struct, such as parser or converter logic, destructure it exhaustively without `..`. Bind used fields and mark intentionally ignored fields with `field: _`.

```rust
let Input {
    data,
    parameters,
    metadata: _,
} = input;
```

Adding or renaming a field then causes a compile error at the broad consumer. Treat that error as a prompt to map, validate, or explicitly ignore the change.

Use direct field access when a method needs only a few fields; exhaustive destructuring should create useful change detection, not noise.

Prefer an exhaustive `match` when several variants or mutually exclusive shapes of one value drive control flow. Avoid stacking `if` or `if let` statements for mutually exclusive cases; one `match` makes exclusivity and future variants compile-visible.

Keep ordinary `if` for independent predicates and guard clauses. Avoid wildcard match arms unless every unlisted variant intentionally has identical behavior.

## Validate Inputs And Fail Loudly On Broken State

Validate every relied-on shape, range, cardinality, ordering, and cross-field relationship of external data once, as it enters the internal representation. For streaming input, validate each record or chunk before downstream stages rely on it.

Return errors at trust boundaries such as parsing, files, network input, and public APIs.

Validate plausible numeric ranges once at the trust boundary. Inside validated code, work from those domain bounds: use natural units, ordinary arithmetic, and the existing overflow behavior. Do not widen integers, convert to unnecessarily fine units, add `checked_*` plumbing, or test impossible magnitudes merely to defend against values the domain cannot produce. Add special handling only when accepted inputs can realistically approach the limit.

For example, express retail planning horizons in planning intervals rather than nanoseconds, and compare domain-bounded `u64` packing-load ratios with `u64` cross-products unless their real bounds require widening.

Assert internal invariants where mutation or transformation could break them. Panic rather than turn an impossible internal state into a plausible fallback result.

Use warnings only for explicitly tolerated defects whose continuation semantics are defined.

## Engineer Numerical And Specialized Kernels

Apply the following guidance when numerical decisions, custom kernels, stochastic behavior, or performance measurements are part of the change.

### Make Numerical Semantics Explicit

Validate numeric input at construction or parsing boundaries. Encode meaningful restrictions in types where doing so removes repeated checks or prevents invalid state.

Keep exact and tolerant operations separate. Define a named comparison policy for each quantity that needs approximation: its accepted finite domain, absolute or relative tolerance formula, and which side of a boundary it favors. Use exact comparison when no such policy is defined.

Use a float wrapper or explicit comparison only where a total order is required. Decide how NaN and infinity are handled before ordering.

Use monotonic proxies such as squared distance when they avoid expensive work without changing the decision. Preserve the exact calculation when the actual value is required.

### Specialize Only Verified Kernels

Keep a clear general implementation until profiling identifies a kernel worth specializing. Limit custom traversal, specialized layout, or forced inlining to the measured kernel.

Retain a small, obviously correct implementation for bounded test inputs as the oracle. Do not keep duplicate production paths unless runtime cross-checking is required; compare specialized results using exact or explicitly tolerant equality.

Do not use `unsafe` as an optimization technique. Treat introducing or expanding unsafe code as forbidden unless the user explicitly authorizes it for the specific change. When authorized, keep the unsafe region small, document the invariant that makes it sound, and leave a check that would fail if the surrounding assumptions drift.

### Measure Behavior

Run computation-heavy workloads with optimization enabled. Use debug builds for fast compile feedback, not production-scale performance conclusions.

Profile the optimized binary. Preserve symbols when the profiler needs them:

```bash
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release
```

Use deterministic seeds, fixed workloads, and recorded configuration for regression measurements. Keep benchmark setup outside the timed kernel and prevent the measured result from being optimized away.

Measure both the isolated kernel and the end-to-end workload. Separate query, update, and mixed behavior when they stress different paths.

For stochastic algorithms, compare distributions of runtime and result quality across repeated runs. Do not trust one favorable outcome.

After every optimization, compare the same correctness boundary and workload before and after. Keep a change only when the measured benefit justifies its complexity.
