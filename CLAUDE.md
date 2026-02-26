# Calce

Financial calculation engine for portfolio tracking.

## Architecture

```
CalcEngine (orchestration) — wires services to pure functions
├── calc/     — pure business logic, no side effects
├── services/ — trait-based data access, in-memory test impls
└── domain/   — data types only, no business logic
```

Domain types are data carriers. Business logic belongs in `calc/`.
Intrinsic operations (e.g. `Money::convert`, `FxRate::invert`) are fine on domain types.

## Comments

Only comment when the comment adds value that the code doesn't already convey.

**Do comment:**
- Non-obvious domain conventions (sign conventions, currency directionality)
- `# Errors` and `# Panics` sections on public functions (required by `clippy::pedantic`)
- Why something exists when the reason isn't obvious (e.g. "Sort for deterministic output")

**Do not comment:**
- `/// Create a new X` — the function is called `new`
- `/// Returns the Y` — the function is called `y()` or `get_y()`
- `/// The Z field` — the field is named `z`
- Module declarations (`pub mod foo`)
- Struct/enum definitions when the name is descriptive
- Enum variants when the variant name + error message are clear

**Rule of thumb:** if the doc comment is just the function/field/type name rephrased as a sentence, delete it.

## Development

```sh
cargo build
cargo test
cargo clippy -- -D warnings
```
