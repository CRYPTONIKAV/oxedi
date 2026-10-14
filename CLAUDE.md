# CLAUDE.md — how work is done in oxedi

Lossless, fast, data-driven EDI 835 parser core in Rust (edition 2024); learning Rust in
depth is a co-equal goal. Public repo `javillegasna/oxedi`. Conversation language is
Spanish; code, commits, PRs and issues are in English.

## Where the truth lives

`.doc/` is local and git-ignored: the maintainers' working notes, never committed. The only
public planning page is `docs/roadmap.md` on the site.

- `.doc/architectural-commitment.md`: the contract. §2 invariants (N1–N7), §3 principles
  (P1–P10), §4 processing model, §5 stage map, §6 decisions (taken T*, open D*), §7 one
  approved section per stage. A stage starts only after its §7 section is approved by the
  owner; a plan is written only after that.
- `.doc/roadmap.md`: stage status, what each unlocks, open decisions by stage.
- `.doc/plans/stage-N-*.md`: executable plans with full code, TDD steps and an exit gate.
- `.doc/state.md`: current snapshot and next steps (update it when a stage changes state).
- `.doc/analysis/`: the owner's study notes with Mermaid diagrams about the
  code (structure, flow, tests and their reasons, Rust concepts, patterns). Update them per
  stage from the real code, not from plans; classDef colours need `color:#111`.
- `docs/roadmap.md` (public, on the site): a short, user-facing summary in English: available
  now, next, being considered; no stage codes, decision ids or internal detail. Update it with
  one line when a stage ships or the next item changes.
- GitHub Project #8 "oxedi835 — Roadmap & Deuda Técnica": deferred review findings as
  issues (Context / Problem / Recommendation, no solutions), Backlog + priority.

## Non-negotiables in code

The code conventions live in one place, `CONTRIBUTING.md#code-conventions`, written for any
contributor: lossless, no panics on input (and its exception for the built-in specs), errors
explain themselves, the structure is data, the sans-IO core and its dependencies, comments,
module layout, file size, positions, names, renames and the commit gates; the test-data rules
are in `CONTRIBUTING.md#tests-and-test-data`. Follow them exactly as written there.
`ARCHITECTURE.md` is the public map of the code. Agent-specific additions:

- The public names map to the contract's codes: "Lossless" is N1, "Errors explain themselves"
  is P10.
- "Comments describe the code, not its history" also excludes stage numbers, principle codes
  (N1, P3) and links to `.doc/` or other planning docs.
- Each new dependency is approved in a §7 section before it is added.
- A stage that renames a public core item or moves a module updates `ARCHITECTURE.md` and
  `crates/oxedi_core/tests/public_paths.rs` in the same commit.

## How a stage runs

1. Discuss open decisions with trade-offs in chat; write the §7 section; owner approves.
2. Write the plan (superpowers:writing-plans); owner approves.
3. Feature branch from `master`. Execute with superpowers:subagent-driven-development.
   The main session only orchestrates: it plans and dispatches, never implements. One
   implementer per task, chosen by complexity: Haiku transcribes complete code from the
   plan, Sonnet handles prose-described or multi-file tasks, Opus takes design judgment.
   The reviewer is one tier above the implementer (Haiku work → Sonnet review, Sonnet
   work → Opus review, Opus work → Opus review). Token budget rule (since Stage 4b): group
   consecutive tasks into batches of two to four for one implementer, review per batch, not
   per task; the plan carries executed code only for design-heavy tasks and precise prose
   for mechanical ones. A whole-branch review on Opus at the end with P10 as an explicit
   focus (triage, not a second audit), one fix wave, one scoped re-review. Rulings go in the
   ledger; the ledger is copied to `.doc/analysis/stage-N-ledger.md` before the workspace
   is deleted.
4. PR against `master`, body in English with two sections, **Intent** and **Verification**;
   never list the diff content; no attribution lines. The owner merges.
5. Deferred minors become issues on Project #8. Then update `.doc/roadmap.md`,
   `.doc/state.md` and, when what users can do changed, `docs/roadmap.md`.

## Commands

`make help` lists every target. The gates and the Python binding have make targets; the raw
commands are kept here because CI runs them directly.

```bash
make gates            # fmt-check, clippy -D warnings, test, bench --no-run, doc -D warnings
make py-test          # maturin develop into .venv, then pytest crates/oxedi_py/tests
make dist && make smoke   # sdist + release wheel, then install in a clean venv outside the repo
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all && cargo fmt --all -- --check
UPDATE_GOLDEN=1 cargo test -p oxedi_core --test engine_golden   # regenerate goldens, then inspect
UPDATE_GOLDEN=1 cargo test -p oxedi_core --test project_golden  # table and diagnostic goldens
cargo bench --workspace                                           # ~5 min; baselines go in commit messages
```

Publishing (`make publish-test`, `make publish`, `make tag`) reads `~/.pypirc`; tokens never
enter the repo. Python floor 3.11; the base wheel declares no Python dependencies.
