---
name: review
description: Review a Rust source file for code quality, structure, performance, and correctness issues. Checks DRY violations, data structure choices, API design, access control consistency, and domain type usage.
argument-hint: <file path or module name to review>
---

You are reviewing Rust code in the Calce financial calculation engine.
The user wants you to review: $ARGUMENTS

## How to review

1. **Read the target file(s).** If the user gave a module name, find the file.
2. **Read its dependencies.** Understand the types, traits, and error types it uses — read the imports, check the domain types, look at how callers use the public API. Use an Explore agent if needed.
3. **Analyze against the checklist below.** Only report real issues — not style nitpicks.
4. **Present findings** as a ranked table (High / Medium / Low) with a short explanation per issue. Lead with the most impactful problems.

## Checklist

### Correctness
- Silent data loss: are values silently dropped, truncated, or coerced? (e.g. summing quantities across different currencies, ignoring enum variants)
- Are error cases distinguishable? Can callers tell "not found" from "unauthorized"?
- Could any method produce wrong results with valid inputs?

### Data structures
- Is a `Vec` used where a `HashMap`/`HashSet` would give O(1) lookup by a natural key?
- Are there `.iter().find(|x| x.id == ...)` or `.iter().filter(|x| x.field == ...)` patterns that scan linearly on a keyed field?
- Are collections sized appropriately for the expected data volume? (Check CLAUDE.md and the crate's module docs for context.)

### DRY / structure
- Are there near-duplicate blocks that differ only in a filter or a single field? Extract shared logic.
- Does the module mix concerns that belong in separate modules? (e.g. SQL in a domain module, business logic in a route handler)
- Are there helper functions that only exist because the caller's data structure is wrong?

### API surface
- Do public methods have a consistent pattern for access control? (All use the centralized check, or all filter — not a mix.)
- Do return types match the caller's needs? (Returning owned `Vec<T>` when callers only iterate; returning `Option` when callers need `Result` with an error reason.)
- Could a method's signature be simplified without losing expressiveness? (e.g. `&str` vs domain type, `i64` vs `AccountId`)

### Domain types
- Are raw `String`/`i64` used where a domain newtype (`UserId`, `InstrumentId`, `Currency`, `AccountId`) exists and would add type safety?
- Are domain types converted to raw types too early, losing compile-time guarantees?
- Are newtypes constructed unnecessarily in hot paths when a reference or borrow would work?

### Performance
- Are collections cloned where a borrow (`&[T]`, `&T`) would suffice?
- Is work repeated per-request that could be done once at load time?
- Are allocations happening in loops that could be hoisted?

### Access control
- Do all methods that return user-specific data enforce access checks?
- Are the checks using the centralized `permissions` module, or are they reimplemented inline?
- Could any public method leak data counts or existence to unauthorized callers?

## Output format

Present a summary table ranked by impact:

| Priority | Issue | Type |
|----------|-------|------|
| **High** | ... | Correctness / Security / ... |
| **Medium** | ... | DRY / Performance / ... |
| **Low** | ... | API / Style / ... |

Then briefly expand on each High and Medium item (1-3 sentences). Skip detailed explanation for Low items — the one-liner in the table is enough.

End with: "Want me to fix any of these?"
