# Canopy Lexicon Dataset

A comprehensive collection of closed-class words and functional lexical items for English natural language processing.

## Overview

The Canopy Lexicon provides structured access to function words, discourse markers, negation patterns, quantifiers, and other linguistic elements that are crucial for semantic analysis but often overlooked by content-focused NLP systems.

## Data Format

The lexicon uses XML format with a well-defined schema (`schema.xsd`) to ensure consistency and enable validation.

### Structure

```
data/canopy-lexicon/
├── schema.xsd              # XML Schema definition
├── english-lexicon.xml     # Main English lexicon data
└── README.md              # This documentation
```

## Word Classes

### 1. Stop Words / Function Words
- **Purpose**: Core stop words that typically carry little semantic meaning
- **Priority**: 10 (highest)
- **Examples**: the, a, an, of, in, on, and, or, but
- **Properties**: 
  - `remove-in-indexing: true`
  - `semantic-weight: 0.1`

### 2. Negation Words
- **Purpose**: Words and patterns that indicate negation or denial
- **Priority**: 9
- **Examples**: not, no, never, nothing, cannot, won't
- **Patterns**: 
  - Prefixes: un-, dis-, in-, im-, ir-, il-, non-
  - Suffixes: -less
- **Properties**:
  - `polarity: negative`
  - `scope-modifier: true`

### 3. Discourse Markers
- **Purpose**: Words that organize discourse and indicate relationships between ideas
- **Priority**: 8
- **Categories**:
  - Temporal: first, then, next, finally, meanwhile
  - Causal: therefore, thus, consequently, because
  - Contrastive: however, nevertheless, although
  - Additive: also, furthermore, moreover
  - Conclusive: overall, ultimately, essentially
- **Properties**:
  - `discourse-function: true`
  - `text-structure: true`

### 4. Quantifiers
- **Purpose**: Words indicating quantity, amount, or degree
- **Priority**: 7
- **Categories**:
  - Universal: all, every, each, both
  - Existential: some, any, several, many, few
  - Numerical: one, two, three, numbers, ordinals
  - Degree: very, quite, extremely, completely
- **Properties**:
  - `quantification: true`
  - `scope-sensitive: true`

### 5. Temporal Expressions
- **Purpose**: Words indicating time, duration, and temporal relationships
- **Priority**: 6
- **Categories**:
  - Time points: now, today, yesterday, tomorrow
  - Duration: always, never, sometimes, often
  - Days: monday, tuesday, wednesday...
  - Months: january, february, march...
  - Relative: before, after, during, until
- **Properties**:
  - `temporal-reference: true`
  - `time-sensitive: true`

### 6. Hedge Words / Uncertainty Markers
- **Purpose**: Words indicating uncertainty, approximation, or hedging
- **Priority**: 5
- **Examples**: perhaps, maybe, possibly, probably, apparently
- **Properties**:
  - `certainty-modifier: true`
  - `epistemic: true`

### 7. Intensifiers
- **Purpose**: Words that strengthen or weaken the force of other words
- **Priority**: 5
- **Categories**:
  - Strong: extremely, incredibly, absolutely, completely
  - Moderate: very, really, quite, fairly
  - Weak: somewhat, slightly, barely, hardly
- **Properties**:
  - `intensity-modifier: true`
  - `scalar: true`

## XML Schema Features

### Word Entry Attributes
- `variants`: Alternative forms (e.g., "n't" for "not")
- `pos`: Part-of-speech tag
- `confidence`: Confidence score (0.0-1.0)
- `frequency`: Usage frequency (if available)
- `context`: Semantic or pragmatic context

### Pattern Matching
- **Pattern Types**: prefix, suffix, infix, whole-word, phrase
- **Regex Support**: Full regular expression patterns for morphological analysis
- **Examples**: Each pattern includes usage examples
- **Confidence Scoring**: Patterns have associated confidence levels

### Properties System
- Extensible key-value properties for each word class
- Type-safe values (string, boolean, integer, float)
- Enables fine-grained linguistic annotation

## Integration with Canopy

This lexicon is designed to integrate with the Canopy semantic analysis pipeline:

1. **Layer 1**: Lexical lookup and classification
2. **Layer 2**: Compositional semantic analysis (negation scope, quantifier scope, discourse structure)
3. **Layer 3**: Pragmatic interpretation (hedge detection, intensity modification)

## Engine Implementation

The lexicon engine will be implemented as `canopy-lexicon` crate, following the same patterns as VerbNet, FrameNet, and WordNet engines:

- **XML Parsing**: Uses canopy-engine XML infrastructure
- **Caching**: High-performance lookup with LRU caching
- **Pattern Matching**: Efficient regex-based morphological analysis
- **Multi-class Support**: Handles overlapping word classifications
- **Confidence Scoring**: Provides uncertainty estimates for classifications

## Usage Examples

### Basic Lexical Classification
```rust
let engine = LexiconEngine::new(config);
let result = engine.classify_word("not");
// Returns: [WordClass::Negation { confidence: 1.0, ... }]
```

### Pattern-based Analysis
```rust
let result = engine.analyze_patterns("unhappy");
// Returns: [Pattern::NegationPrefix { prefix: "un", confidence: 0.8, ... }]
```

### Discourse Analysis
```rust
let result = engine.analyze_discourse("However, the results were significant.");
// Returns: [DiscourseMarker::Contrastive { word: "however", ... }]
```

## License

MIT License - Same as the main Canopy project.

## Contributing

To add new words or patterns:

1. Update `english-lexicon.xml` following the schema
2. Validate against `schema.xsd`
3. Add appropriate confidence scores and examples
4. Update this documentation

## Future Extensions

- **Multilingual Support**: Additional language lexicons
- **Domain-Specific Lexicons**: Technical, medical, legal terminology
- **Dynamic Learning**: Machine learning-based pattern discovery
- **Contextual Classification**: Context-dependent word class assignment