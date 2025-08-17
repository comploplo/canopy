# VerbNet Patterns for canopy.rs

This file documents the VerbNet-based verb patterns ported from the Python V1 system. These patterns will be implemented in Rust for M3 (Event Structure milestone).

## Pattern Structure

Each pattern maps syntactic structures to theta roles with confidence scores:

```rust
struct VerbPattern {
    lemma: String,
    syntactic_pattern: String,
    role_mapping: HashMap<String, ThetaRole>,
    confidence: f32,
    description: String,
}
```

## Core Pattern Categories (from Python V1)

### 1. Transfer Verbs (give, send, show, tell, teach, offer, hand)

- **Pattern**: `subj_obj_iobj` → Agent gives Theme to Recipient
- **Pattern**: `subj_obj_pp_to` → Agent gives Theme to Recipient (PP)
- **Confidence**: 0.95

### 2. Motion Verbs (go, come, move, travel, walk, run, drive)

- **Pattern**: `subj_pp_to` → Theme moves to Goal
- **Pattern**: `subj_pp_from` → Theme moves from Source
- **Confidence**: 0.9

### 3. Psychological Verbs (like, love, hate, fear, enjoy, prefer, admire)

- **Pattern**: `subj_obj` → Experiencer feels about Stimulus
- **Confidence**: 0.95

### 4. Creation Verbs (make, build, create, construct, produce, manufacture)

- **Pattern**: `subj_obj` → Agent creates Patient
- **Pattern**: `subj_obj_pp_with` → Agent creates Patient with Instrument
- **Confidence**: 0.9

### 5. Communication Verbs (say, speak, talk, discuss, mention, announce)

- **Pattern**: `subj_obj` → Agent communicates Theme
- **Pattern**: `subj_pp_to` → Agent communicates to Recipient
- **Confidence**: 0.85

## Implementation Notes

1. **No External XML**: Python V1 uses hardcoded patterns, not external VerbNet XML files
2. **Pattern-Based**: Focus on syntactic patterns rather than exhaustive verb lists
3. **Confidence Scoring**: Each pattern has a confidence score for backoff strategies
4. **Lexicon-Free**: Emphasizes patterns over memorized verb-role mappings

## Rust Implementation Plan (M3)

```rust
// crates/canopy-semantics/src/verbnet.rs
pub struct VerbNetPattern {
    pub lemma: String,
    pub syntactic_pattern: SyntacticPattern,
    pub role_mapping: HashMap<ArgumentPosition, ThetaRole>,
    pub confidence: f32,
    pub description: String,
}

pub enum SyntacticPattern {
    SubjObj,
    SubjObjIobj,
    SubjObjPpTo,
    SubjObjPpWith,
    SubjPpTo,
    SubjPpFrom,
}

pub enum ArgumentPosition {
    Subject,
    Object,
    IndirectObject,
    PrepositionalPhrase(Preposition),
}
```

This will be implemented in M3: Event Structure milestone as part of the theta role assignment system.
