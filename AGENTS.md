# AGENTS.md: working agreement for hyperpace

Every coding agent (Claude Code, and any tool that reads AGENTS.md) follows this contract in this repo. Stacks: none yet. Run the gate with vertexia gate before pushing. The research-to-implementation workflow is in the vertexia-research / vertexia-plan / vertexia-implement / vertexia-loop skills.

# Doctrine

The generic engineering mental model, adopted across every repo. Repo-specific
examples are the pattern to apply, not literal scope.

## The mental model

A handful of instincts. Internalize these and the specific rules follow.

- Honesty over polish. A true "not measured yet" beats a pretty number that is
  not real. Never once fabricate a result. This value outranks the others.
- Ask before you implement. Decisions are the operator's. Surface options, get
  the call, then build. Do not decide scope, exclude a feature, or pick an
  approach on your own.
- Done means done. No stubs, no `todo!()`, no halfway state, no quietly dropped
  feature. If something is deferred, it is written down honestly, never hidden.
- Bound the resource that actually fails, not a proxy for it. Counting is not
  bounding bytes. Find the real failure resource (RAM, time, fan-out, money)
  and bound that, streaming so you hold a page and never the whole set.
- Everything is highly performant. "It works" is not done. Done includes fast,
  and the number is in the PR.
- When two places must agree, they call one function. Two copies are a
  divergence waiting to ship.
- Reach for the mechanism already here. Extend the existing pattern; do not
  invent a parallel one.
- The least code that does the job. Climb the ladder before writing (need it at
  all, the standard library, a native feature, an installed dependency, one
  line) and stop at the first rung that holds. Deletion beats addition. Lazy is
  efficient, never negligent: it trims speculative scope, never the validation,
  the error handling, the security, or the honesty fence.
- Fail loud, with the reason. A wedged or unknowable state surfaces itself with
  an actionable message, never a false green.
- Verify the real artifact, not a proxy. A green pipeline is not a deploy.

## 0. Hard rules

Several are enforced by hooks that block the action. Honor them regardless.

1. No AI attribution, anywhere. Never a `Co-Authored-By: Claude` (or any
   Claude / Anthropic) trailer, never a "Generated with Claude Code" or
   "Generated with" footer, not in commits, not in PR or issue bodies, not in
   files. Never change `git config user.email` to work around a squash adding
   a co-author line. A real human co-author trailer is fine.
2. No em-dash and no en-dash, anywhere: chat, code, comments, commit messages,
   PR text, websites, docs, scaffolded templates, generated content. Use a
   plain ASCII hyphen, or restructure with a colon, comma, or parentheses.
   Arrows (->) are fine. Reason: they read as AI-written and are wrong in a
   real codebase.
3. Ask before implementing. Everything is asked, even in the plan stage. No
   unilateral scope cut, no random feature exclusion, no approach picked alone.
   Present choices (a skill prompt or a multiple-choice), take the operator's
   answer, then act. The "proceed without confirmation, continue every time"
   instinct is erased here. The one exception is an explicit loop run
   (vertexia-loop) the operator starts: it is pre-authorized to run to
   completion without stopping, and it records every decision in
   docs/decs/<plan>_DECS.md for review, so it is never silent.
4. Never commit or push without being told. "Commit regularly" does not apply.
   Make the change, run the gate, surface what you did, then wait for the go.
   A commit or push hold may be active (see section 2): respect it.
5. No stub implementations. Merged code is fully implemented, fully tested,
   fully documented, or it is a clearly-labelled pre-implementation placeholder
   tied to a design doc. There is no intermediate state. Deferred work is
   documented honestly, not hidden.
6. Run JavaScript and TypeScript sealed in a sandbox. Never invoke node, npx,
   tsx, or ts-node directly. Run them sealed (no net, no fs, no env), granting
   a capability only when genuinely required, narrowly, per run. Package-manager
   scripts (npm run, npm test) are fine.
7. Report outcomes faithfully. Tests failed: say so with the output. Step
   skipped: say that. "Done and verified" only when it is. This is section 8.

## 1. How to work

- Design package before code. Non-trivial work starts with a written package
  (problem, approaches, scope, deferred, plan) that the operator approves
  before any product code. Then phased implementation, de-risk experiments
  first.
- Batch edits, gate once. Make the related changes, then run the slow checks
  once at the end. Do not loop compile-and-test after every edit, especially
  when the suite is slow.
- Mirror the code already there. Match naming, error handling, comment density,
  and idiom. A new pattern is a cost; introduce one only when the existing
  pattern cannot carry the change.
- Navigate with the code index, and keep it synced. vertexia init registers the
  index as the vertexia-code MCP server, rooted at the top level of the directory
  you are working in. Query it with the code_explore / code_search / code_callers
  / code_callees / code_impact / code_show / code_outline tools (or the
  equivalent vertexia code ... CLI) to locate and understand code before a broad
  grep or re-reading whole files: it returns a focused bundle and saves context.
  Send code_sync against that top level regularly: at the start of a session,
  after each batch of edits, after a subagent or a workflow writes code, and
  before any broad navigation. Two hooks back this up in an initialized repo, a
  SessionStart sync and a PostToolUse sync after every Edit and Write, so a
  forgotten sync costs you a stale count in your own report, never a stale
  answer. Read tools refresh before they answer; code_sync is how you see what
  the index actually covers now.
- Orchestrate with workflows. For substantial multi-step work (a multi-file
  feature, an audit, a migration, a broad sweep), drive it as a workflow of
  parallel subagents, not one long serial pass. Decompose, fan out, verify.
- Tier the models: reason on the latest model, code on Sonnet. Run planning,
  research, design, review, and every engine decision on the latest, most-capable
  model with maximum reasoning effort. This is never pinned: today that is Opus
  4.8 (max reasoning); when a more capable model ships (for example Fable 5),
  switch the reasoning tier to it. Delegate only the writing of code to the latest
  Sonnet (the vertexia-coder agent, model sonnet): hand it a well-specified change
  and let it implement. Reasoning subagents inherit the capable model; never
  downgrade them. Keep the capable model for the thinking, not for mechanical
  code-writing.
- Ship concrete artifacts, not advice. A working file, a runnable script, a
  passing test, over a description of what could be done.
- Push back with evidence. If a request is wrong or risky, say so and show why:
  a file, a line, a failing case. Do not silently comply, do not refuse without
  proof.
- When the operator is thinking out loud, the deliverable is your assessment, a
  diagnosis, not an immediate change. Change code when asked.
- Before deleting or overwriting, read the target. If it contradicts how it was
  described, or you did not create it, surface that instead of proceeding.
- Use the latest toolchain. Update proactively (compiler, linters, libraries).
  Expect a new lint set on a bump and clear it. Keep the supported floor only
  where a dependency requires it.

## 2. Git and change hygiene

- One-liner commit subjects. A compact conventional subject (fix(area):,
  feat(area):, docs:, ci:). No multi-paragraph body. Detail lives in code,
  docs, or the PR description. A body is only for a revert or a migration note,
  and then 10 lines or fewer.
- `git commit -m "..."`, never `-F <file>`. The temp-file pattern causes
  stale-buffer bugs.
- One logical change per commit. Do not smuggle a cleanup into a feature commit.
- Commit and push hold. The operator can disable committing and pushing for a
  window (research and exploration). While a hold is active, do not commit or
  push: ask, or offer a multiple-choice (commit now / keep holding / extend the
  hold / push). The hold is hook-enforced; a held commit or push is denied with
  the reason and the time remaining.

## 3. Code quality

### The least code that works

Before writing, climb the ladder and stop at the first rung that holds:

1. Does this need to exist? A speculative need is skipped and said in one line.
2. Does the standard library do it? Use it.
3. Does a native platform feature cover it? A DB constraint over app code, CSS
   over JS, a built-in over a library.
4. Does an already-installed dependency solve it? Use it; never add a new
   dependency for what a few lines do.
5. Can it be one line? One line.
6. Only then: the minimum code that works.

The ladder is a reflex, not a research project: two rungs hold, take the higher
one and move on. Deletion beats addition, boring beats clever, the shortest
working diff wins. No abstraction with one implementation, no factory for one
product, no config for a value that never changes, no scaffolding for a later
that can scaffold itself.

Concision is subordinate to the floor. Never simplify away input validation at a
trust boundary, error handling that prevents data loss, a security measure,
accessibility, the honesty fence (section 8), the tests of non-trivial logic
(section 6), or anything explicitly asked for. Lazy is efficient, not negligent.
Two options of equal size: take the one that is correct on the edge cases.

Mark a deliberate simplification with a `vertexia:` comment that names its
ceiling and the upgrade path, so a shortcut reads as intent and a deferral
cannot rot silently: `// vertexia: global lock, per-account locks if throughput
matters`. These markers are part of the deferred-work record (rule 5); the
vertexia-debt skill harvests them into a ledger.

- One canonical implementation per concept. No copy-paste across files, layers,
  or languages; factor out by the third occurrence. The sharp form: when two
  call sites must agree on a decision, they call one shared function, never two
  copies that drift. Search for an existing pattern to extend before adding.
  When the repetition is structural (many near-identical blocks differing only
  in a few tokens, like a registry of similar entries), collapse it with the
  language's metaprogramming: a Rust `macro_rules!`, not copied variants.
- No source file over ~1000 lines. A file that reaches the limit is modularized:
  split it along logical seams into multiple focused files (in Rust, a module
  per file via mod; the same idea in every language), never grown past the
  limit. New code lands in a focused module, not appended to an oversized file.
  This applies to every file, including the code that builds vertexia itself.
- No panics on a production path. No `unwrap()` / `expect()` / `todo!()` /
  `unimplemented!()` reachable from a public API or a request path. Propagate
  and handle errors with structured types. Tests may assert.
- No silently swallowed errors. Every error is propagated, mapped, or handled
  out loud. A failure path emits the same audit, log, and metric a success does.
- Idiomatic first. Iterators over manual loops, early-return over deep nesting,
  the standard library over a hand-roll, a borrow over a clone, a domain type
  over a bare String where it prevents a mistake. Idiomatic is the fast form,
  not a stylistic nicety.
- Lean comments. Reserve a comment for a genuinely non-obvious why (a subtle
  invariant, a workaround, a spec reference). Do not narrate what the code says
  or doc every field. Prefer a self-explanatory name over a comment.

## 4. Concurrency

- Lock-free by default; a blocking lock is a last resort. A `Mutex<HashMap>`
  where a concurrent map or a typed channel or actor would serve is a smell.
  Predictable latency under contention is the requirement. The operator's house
  toolkit for this is the kovan ecosystem (kovan-map, kovan-channel,
  kovan-queue) plus plain atomics for shared state; reach there first.
  Decision rule: multiple threads share it, use a lock-free structure;
  single-threaded or process-local, plain std is correct and idiomatic.
- Async is cancel-safe. Cancellation at any await leaves observable state
  consistent. The only acceptable wait is awaiting a channel or future.
- Make Send and Sync explicit on public concurrency-bearing traits.

## 5. Performance and scale

Everything must be highly performant. Reviewers reject changes that violate
this the same way they reject a correctness bug.

- Stream large results; never materialize the whole set. Anything that scales
  with input is bounded and streamed: push the LIMIT to the source, emit rows
  as produced, hold a page not the universe.
- Bound by the quantity that actually fails. A limit on a proxy (a count) does
  not bound the real resource (bytes, RAM). Page reads, do not load a whole
  dataset to serve a few rows or one item.
- One boundary crossing per batch, never per row. The boundary (an engine call,
  a serialize, a syscall) dominates. New surfaces are batch-first; per-row is
  sugar over the batch path.
- Native for O(n) byte work, interpreted only for logic. Codecs, hashing,
  compression, framing belong in the fast path, not a per-byte interpreted loop.
- Bound everything input-sized: retries (backoff plus a max), queues, buffers,
  fan-out. An unbounded anything is an incident waiting for traffic.
- Measure or it did not happen. A perf claim ships with a script and
  before-and-after numbers in the PR. Budgets are re-tightened when a fast path
  lands, never abandoned.

## 6. Tests

- Production-grade, colocated unit tests; integration tests alongside. Every
  public behavior has at least one test, edge cases and failure paths included.
  No test that passes when the logic is wrong.
- Property tests for round-trips, idempotence, ordering, and invariants.
- No fixed `sleep()` in a polling loop. Use a deadline plus bounded backoff so a
  fast machine finishes instantly and a slow one still passes. A flaky test is a
  failing test. Set the test thread count for CI parity where it matters.
- Machine-epsilon tolerance for approximate or boundary float checks; byte and
  bit exact assertions stay exact.
- Local-first. The thing runs locally with zero external services by default;
  external backends are opt-in. Test locally, end to end, before deploying.
- Concision (section 3) trims speculative production code, not coverage. A lazy
  implementation of non-trivial logic still ships the check that fails when it
  breaks; YAGNI applies to a speculative test, never to the floor.

## 7. Documentation and observability

- Document every public item; give each module a one-paragraph what-this-is and
  what-invariants-it-holds. Cross-reference design and incident docs by name.
- Diagrams are standalone `.mermaid` files (raw mermaid source), not a fenced
  block inside a markdown doc. Keep prose in a companion `.md` that points to
  the diagram; never embed a second copy (drift risk).
- Structured logging, not string-formatted. Never log PII or payload content;
  reference an id. A counter per error variant and state transition, a
  histogram per latency-sensitive op, a health endpoint on every long-running
  binary.

## 8. The honesty fence

The credibility of the work lives here. A breach is a Critical bug.

- Never fabricate a number or a status. Unmeasured is "not measured yet", never
  a plausible default. A 0 meaning "no data" must not render as "0%".
- Fail loud, not open. "If we cannot compute it, return PASS" is prohibited. The
  safe default is a visible "not yet" or "unknown", never a false green. A
  wedged or orphaned job surfaces the reason it is stuck.
- No silent caps. If you truncate, sample, or bound coverage, say so where the
  result shows. Silent truncation reads as complete when it is not.
- Do not over-claim. An absolute guarantee ("100% reliable", "always") is
  reserved for what is actually guaranteed (a deterministic path), not a
  statistical bound. Say what the number is and what it is not.
- Do not leak. Internal codenames, raw internal metrics, and competitor or
  external product names stay out of customer-facing copy and out of code.

## 9. UI minimalism

- The design speaks for itself. Layout, components, and concise labels carry the
  meaning, not paragraphs. One subtitle line at most; empty, loading, and error
  states are short and informative. Reuse the established primitives.

## 10. Data and migrations

- Migrations are append-only; never edit an applied one. A checksum mismatch on
  an applied migration crash-loops the next deploy. Add a new forward migration,
  idempotent (IF NOT EXISTS, INSERT ... ON CONFLICT, UPDATE ... WHERE). Exclude
  the migrations dir from repo-wide text sweeps; even a comment edit changes the
  checksum.

## 11. Security

- Validate untrusted input at the trust boundary, once, explicitly.
- Vetted crypto only; never roll your own. Secrets come from a secret manager,
  never committed files, shared-box env vars, or logs.
- No PII or secrets in logs or error messages.

## 12. Deploy and infrastructure

- A green pipeline is not a live deploy. Confirm the artifact that serves
  traffic changed: trace the running image or bundle back to the commit. A
  health endpoint that does not run the new code proves nothing.
- Infra and other outward, hard-to-reverse actions (a terraform apply, a DNS
  change) are the operator's to run unless explicitly delegated.
- Document every infrastructure change in the repo it is applied to. The how
  must be recoverable: what changed, the exact commands run to apply it, the env
  vars read or set and the check that each is present and correct, the
  verification that it took, and the rollback. Keep an infra change log
  (docs/infra/, one dated entry per change, newest first; see
  templates/infra-change.md). Never apply an infra change you cannot document,
  and never skip the log. A change whose how is not written down did not happen
  safely. This rule is not optional.

## 13. The gate

Make the gate green before asking to push. `vertexia gate` detects the repo's
stacks and runs each one's checks. Per stack, at minimum:

- format check (no diff; use --check, never auto-rewrite a vendored tree),
- lint, warnings as errors,
- doc build, warnings as errors (broken doc links fail CI; this is the most
  forgotten step and it belongs in the gate),
- the test suite,
- stack extras when configured: coverage threshold, dependency and license
  audit, type-check, frontend build, migrations-checksum check.

When a faster sub-agent or a parallel task writes the code, it is not done
because it is green. Read the real diff, re-run the full gate on the touched
area, confirm tests were actually added.

## 14. The workflow

vertexia drives work to implementation as gated phases. The short form:

  [research] -> plan (a package) -> approve -> implement (phased) -> verify

Each boundary is an ask. Research is optional: run it for novel or
performance-critical work where the literature decides the approach; skip it
when the operator already has the knowledge and directs the plan. The planning
engine writes the package either way; implementation is phased and asks before
each step; verification proves the claim. Detail is in docs/WORKFLOW.md and the
vertexia-research / vertexia-plan / vertexia-implement / vertexia-loop skills.
Gate granularity (every-decision, phase-gate, or hold) is set per run.
