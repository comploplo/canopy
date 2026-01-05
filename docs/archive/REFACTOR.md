# Canopy Refactor Engineering Plan

## Goals

- **Reduce cognitive load** by collapsing 11 "concept crates" into modules where crate boundaries aren't providing value
- **Establish a hard dependency wall** so the semantic kernel can't silently re-couple to heavy datasets
- **Enable golden snapshot testing** via stable, deterministic input IR + output schema
- **Maintain extensibility** for future work (incrementality, ambiguity, QUD) without requiring redesign

## Non-Goals (This Phase)

- No "theory mode" productization work—this is code hygiene + architecture enforcement
- No large behavioral changes to semantics—primarily repackaging + interfaces

______________________________________________________________________

## Target Architecture: 3 Crates

### 1. `canopy` (facade + kernel library)

The public contract crate. What users import for "Canopy semantics."

**Exposes:**

- `core` boundary types (IDs, spans, IR)
- Provider traits (what runtime/resources must supply)
- Kernel entrypoints (semantic composition + discourse update)
- Small curated lexicon lists (definitional, stable)

**Must NOT** depend on dataset loaders.

### 2. `canopy-resources` (heavy loaders + adapters)

The only place heavy coupling is allowed.

**Owns:**

- Loading VerbNet/FrameNet/PropBank/WordNet
- Caching, indexing, memory maps
- Provider implementations that satisfy `canopy` traits
- Tokenization/parsing integration

Swappable later for different/smaller resources.

### 3. `canopy-cli` (binary)

Composition root. Keeps CLI deps (clap, tracing config, filesystem) out of library crates.

**Depends on both** `canopy` and `canopy-resources`—this is correct and intended.

______________________________________________________________________

## Target Directory Layout

### `crates/canopy/src/`

```
lib.rs                     # re-export facade: analyze(), analyze_with()
core/
  mod.rs
  ids.rs                   # EntityId/EventId/TokenId/etc.
  span.rs                  # Span / offsets
  ir/
    mod.rs
    syntax.rs              # AnnotatedSyntax IR (contract)
    features.rs            # morph/tense/aspect flags
  lexicon.rs               # small curated wordlists + helpers (hardcoded)
kernel/
  mod.rs
  events/
    mod.rs                 # event composition logic
    compose.rs
    roles.rs
  discourse/
    mod.rs                 # DRT + binding
    drt.rs
    binding.rs
    salience.rs
runtime/
  mod.rs
  provider.rs              # SyntaxProvider, SenseProvider, RoleProvider, etc.
  coordinator.rs           # orchestration glue
  config.rs
output/
  mod.rs
  schema.rs                # CanopyIR types
  normalize.rs             # stable ordering + stable IDs for snapshots
  pretty.rs                # pretty-print LF/DRS
tests/
  golden/                  # input fixtures + expected normalized output
```

### `crates/canopy-resources/src/`

```
lib.rs                     # Resources handle + provider impls
config.rs
datasets/
  mod.rs
  verbnet.rs
  framenet.rs
  propbank.rs
  wordnet.rs
  loaders.rs               # IO/cache/versioning
lexicon/
  mod.rs
  entries.rs               # large file-derived lexicon tables
  lookup.rs
tokenize/
  mod.rs
treebank/
  mod.rs
adapters/
  mod.rs
  ud.rs                    # parser/tagger integration
rules/
  mod.rs
  mapping.rs               # sense/frame → kernel inputs
```

______________________________________________________________________

## Crate Migration Mapping

| Current Crate                             | Destination                                              | Action                                                           |
| ----------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------------- |
| `canopy-cli`                              | `canopy-cli`                                             | **Keep.** Rewire to depend on new `canopy` + `canopy-resources`. |
| `canopy-core`                             | `canopy::core`                                           | **Migrate.** IDs, spans, IR become modules.                      |
| `canopy-events`                           | `canopy::kernel::events`                                 | **Migrate then delete.** Pure composition/event building.        |
| `canopy-discourse`                        | `canopy::kernel::discourse`                              | **Migrate then delete.** DRT + binding + salience.               |
| `canopy-engine`                           | **Split:** `canopy::runtime` + `canopy-resources::rules` | Resource-backed mapping → resources. Orchestration → runtime.    |
| `canopy-pipeline`                         | **Becomes** `canopy`                                     | Best candidate to repurpose as the new facade crate.             |
| `canopy-semantic-engines`                 | **Becomes** `canopy-resources`                           | Already has the right shape. Rename package.                     |
| `canopy-tokenizer`                        | `canopy-resources::tokenize`                             | **Migrate.** Fold into resources.                                |
| `canopy-tokenizer/crates/canopy-framenet` | `canopy-resources::datasets::framenet`                   | **Delete nested crate.** Fold code upward.                       |
| `canopy-treebank`                         | `canopy-resources::treebank`                             | **Migrate.** Keep useful benches/examples inside resources.      |

______________________________________________________________________

## Design Decisions (Architecture Review 2026-01-02)

### Kernel Purity Principles

1. **Provider returns decomposition** - SenseProvider returns `PredicateDecomposition` with LittleVType, expected_roles, sub_events. Kernel receives pre-decomposed structures.

1. **No word-level knowledge in kernel** - No lemma→LittleV mappings, no VerbNet class patterns in kernel. Resources implement all word-level logic.

1. **UTAH mappings OK** - Basic UD→theta role mappings (nsubj→Agent, obj→Patient) based on Baker's Uniformity of Theta Assignment Hypothesis are linguistic universals, fine to keep in kernel.

1. **Type bridging at boundaries** - `crate::core` is new clean types, `canopy_core` is legacy. Kernel uses `crate::core` internally, converts at provider call sites.

1. **Kernel owns DRS** - DRS construction is core theory. Kernel builds DRS from events + binding.

1. **Mock provider testing** - Kernel tests use mock providers returning predetermined decompositions. No real words in kernel tests.

### Implementation Strategy

**Fresh implementations, not code migration.** Reference legacy crates for behavior, implement clean.

______________________________________________________________________

## Provider Trait Interfaces (Revised)

```rust
// canopy/src/runtime/providers.rs

/// Structured return type for predicate decomposition
#[derive(Debug, Clone)]
pub struct PredicateDecomposition {
    pub sense_id: SenseId,
    pub little_v_type: LittleVType,
    pub expected_roles: Vec<ThetaRole>,
    pub sub_event: Option<Box<PredicateDecomposition>>,
    pub confidence: f32,
}

pub trait SyntaxProvider: Send + Sync {
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError>;
}

/// Returns fully decomposed predicate structures (not just sense IDs)
pub trait SenseProvider: Send + Sync {
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError>;
}

/// Maps syntax arguments → thematic roles, conditioned on sense
pub trait RoleProvider: Send + Sync {
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError>;
}

/// Discourse connective detection
pub trait DiscourseCueProvider: Send + Sync {
    fn is_discourse_connective(&self, token_id: TokenId, syntax: &AnnotatedSyntax) -> bool;
}

/// Ergonomic supertrait for production wiring
pub trait CanopyProvider:
    SyntaxProvider + SenseProvider + RoleProvider + DiscourseCueProvider {}

impl<T> CanopyProvider for T where
    T: SyntaxProvider + SenseProvider + RoleProvider + DiscourseCueProvider {}
```

**Key benefits:**

- Provider returns full decomposition → kernel is pure
- Fine-grained traits → easy unit tests with mocks
- `CanopyProvider` supertrait → ergonomic for production

______________________________________________________________________

## Kernel Entrypoint Shape

```rust
// canopy/src/lib.rs or kernel/mod.rs

pub struct Kernel { /* rule config */ }

impl Kernel {
    pub fn analyze_sentence(
        &self,
        syn: &AnnotatedSyntax,
        provider: &impl crate::runtime::CanopyProvider,
        ctx: &mut crate::kernel::discourse::Context,
    ) -> anyhow::Result<crate::output::Analysis> {
        // 1) choose senses (or accept pre-annotated)
        // 2) compose events
        // 3) binding + discourse update
        // 4) produce output
    }
}
```

This keeps the kernel pure—no dataset imports inside `canopy`.

______________________________________________________________________

## Resources Crate Shape

```rust
// canopy-resources/src/lib.rs

pub struct Resources { /* indices, caches */ }

impl Resources {
    pub fn load(cfg: &ResourcesConfig) -> Result<Self> { ... }
}

impl canopy::runtime::SyntaxProvider for Resources { ... }
impl canopy::runtime::SenseProvider for Resources { ... }
impl canopy::runtime::RoleProvider for Resources { ... }
impl canopy::runtime::DiscourseCueProvider for Resources { ... }
```

Everything else stays behind modules (`datasets::*`, `tokenize::*`, `rules::*`).

______________________________________________________________________

## Boundary Enforcement

### cargo-deny Configuration

```toml
# deny.toml (workspace root)
[bans]
deny = [
  { crate = "canopy-resources", wrappers = ["canopy-cli"], reason = "Resources must not leak into the library crate." }
]
```

If `canopy` tries to depend on `canopy-resources`, CI fails.

### Backup CI Check

```bash
# In CI script
cargo tree -p canopy | grep canopy-resources && exit 1 || exit 0
```

Catches issues even without running `cargo deny`.

______________________________________________________________________

## Migration Plan

### Step 0: Freeze Behavior with Golden Snapshots

Before any structural changes:

1. Pick ~20 sentences (include 2-sentence discourses)
1. Add output normalization step (`Analysis::normalize_for_snapshot()`)
1. Store expected outputs under `crates/canopy/tests/golden/`

Normalization should provide:

- Stable ID assignment (positional IDs fine for snapshots)
- Stable ordering of lists/maps
- Stripped timestamps / nondeterministic fields

### Step 1: Create New Crates (Without Deleting Old)

**Do NOT "clear out canopy-core" in place.** That breaks everything.

1. **Repurpose `canopy-pipeline` → `canopy`** (rename package in Cargo.toml)
1. **Repurpose `canopy-semantic-engines` → `canopy-resources`** (already has the right shape)
1. Keep `canopy-cli` as-is

Now you have a stable "new path" that can grow while old crates exist.

### Step 2: Wire CLI to New Facade

Update `canopy-cli` to:

- Depend on both `canopy` and `canopy-resources`
- Use `Resources::load(cfg)` + `canopy::analyze_with(&resources, text, cfg)`

### Step 3: Migrate by Module Slices

Order that minimizes breakage:

1. **Move curated wordlists** → `canopy::core::lexicon`
1. **Move output schema + normalize** → `canopy::output`
1. **Move `canopy-events`** → `canopy::kernel::events`
1. **Move `canopy-discourse`** → `canopy::kernel::discourse`
1. **Split `canopy-engine`** → runtime vs resources rules
1. **Absorb `canopy-tokenizer`** → `canopy-resources::tokenize`
1. **Absorb `canopy-treebank`** → `canopy-resources::treebank`
1. **Delete nested `canopy-framenet`** → fold into `canopy-resources::datasets::framenet`

After each slice: run golden tests, then delete/deprecate the old crate.

### Step 4: Add Boundary Enforcement

Once migration complete:

- Add `deny.toml` configuration
- Add CI tree check
- Verify `canopy` builds without datasets present

______________________________________________________________________

## Acceptance Criteria

- [ ] `canopy` crate builds and tests **without any dataset downloads present**
- [ ] `canopy-cli` is the **only crate** that needs datasets
- [ ] Golden snapshots pass deterministically on clean checkout
- [ ] `cargo tree -p canopy | grep canopy-resources` returns empty
- [ ] Provider traits enable unit testing kernel with stub implementations
- [ ] Total crate count reduced from 11 → 3 (+ optional future LSP/MCP crates)

______________________________________________________________________

## Future Crates (Only If Earned)

- `canopy-lsp` — if LSP implementation becomes non-trivial
- `canopy-mcp` / agent skill crate — if MCP server becomes substantial
- `canopy-wasm` — if browser/WASM target needed

These should only exist if they have unique dependency requirements or deployment targets.
