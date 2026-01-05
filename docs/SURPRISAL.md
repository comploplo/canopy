# SurprisalModel Integration

Canopy uses information-theoretic surprisal for disambiguation and processing difficulty prediction. This document covers the `SurprisalModel` trait and how to implement custom models.

## Overview

### Surprisal Theory

Surprisal measures how unexpected a word is given its context:

```
S(word) = -log₂ P(word | context)
```

| Surprisal | Probability | Interpretation                          |
| --------- | ----------- | --------------------------------------- |
| 0 bits    | 100%        | Completely predictable                  |
| 1 bit     | 50%         | Coin flip                               |
| 3.3 bits  | 10%         | Somewhat unexpected                     |
| 6.6 bits  | 1%          | Unexpected                              |
| 10+ bits  | \<0.1%      | Very surprising (garden-path indicator) |
| 20 bits   | ~0.0001%    | Essentially unseen                      |

High surprisal predicts processing difficulty (Hale 2001, Levy 2008).

## The SurprisalModel Trait

```rust
pub trait SurprisalModel: Send + Sync {
    /// P(token | prefix) - core probability estimate
    fn word_probability(&self, token: &str, prefix: &[&str]) -> f64;

    /// P(sense | word, context) - for sense disambiguation
    fn sense_probability(&self, sense_id: &str, word: &str, context: &[&str]) -> f64;

    /// Surprisal in bits (default: uses word_probability)
    fn word_surprisal(&self, token: &str, prefix: &[&str]) -> Surprisal {
        let p = self.word_probability(token, prefix);
        if p <= 0.0 {
            Surprisal::from_bits(20.0)  // Cap for unseen words
        } else {
            Surprisal::from_probability(p)
        }
    }

    /// Joint probability P(w1, w2, ..., wn)
    fn reading_probability(&self, words: &[&str]) -> f64 { /* ... */ }

    /// Total surprisal for sentence
    fn sentence_surprisal(&self, words: &[&str]) -> Surprisal { /* ... */ }
}
```

**Requirements**:

- Must be `Send + Sync` (thread-safe)
- `word_probability` and `sense_probability` are required
- Other methods have default implementations

## Reference Implementation

`UniformSurprisalModel` provides a baseline with equal probabilities:

```rust
use canopy::UniformSurprisalModel;

let model = UniformSurprisalModel {
    vocabulary_size: 50_000,  // Typical English vocabulary
    sense_count: 5,           // Average senses per word
};

// Every word has P = 1/50,000 ≈ 0.00002
let p = model.word_probability("cat", &["the"]);
// Surprisal ≈ 15.6 bits

// Every sense has P = 1/5 = 0.2
let sp = model.sense_probability("cat.n.01", "cat", &["the", "black"]);
```

**When to use**: Testing, establishing baselines, when no real LM is available.

## Implementing Custom Models

### N-gram Model Example

```rust
use canopy::kernel::incremental::{SurprisalModel, Surprisal};
use std::collections::HashMap;

pub struct BigramModel {
    unigram_counts: HashMap<String, u64>,
    bigram_counts: HashMap<(String, String), u64>,
    total_unigrams: u64,
    smoothing: f64,  // Add-k smoothing
}

impl BigramModel {
    pub fn from_corpus(sentences: &[Vec<String>]) -> Self {
        let mut unigram_counts = HashMap::new();
        let mut bigram_counts = HashMap::new();
        let mut total = 0u64;

        for sentence in sentences {
            let mut prev = "<s>".to_string();
            for word in sentence {
                *unigram_counts.entry(word.clone()).or_insert(0) += 1;
                *bigram_counts.entry((prev.clone(), word.clone())).or_insert(0) += 1;
                prev = word.clone();
                total += 1;
            }
        }

        Self {
            unigram_counts,
            bigram_counts,
            total_unigrams: total,
            smoothing: 0.1,
        }
    }
}

impl SurprisalModel for BigramModel {
    fn word_probability(&self, token: &str, prefix: &[&str]) -> f64 {
        let prev = prefix.last().map(|s| s.to_string()).unwrap_or_else(|| "<s>".to_string());
        let vocab_size = self.unigram_counts.len() as f64;

        // Bigram with add-k smoothing
        let bigram_count = self.bigram_counts
            .get(&(prev.clone(), token.to_string()))
            .copied()
            .unwrap_or(0) as f64;

        let prev_count = self.unigram_counts
            .get(&prev)
            .copied()
            .unwrap_or(0) as f64;

        (bigram_count + self.smoothing) / (prev_count + self.smoothing * vocab_size)
    }

    fn sense_probability(&self, _sense_id: &str, word: &str, context: &[&str]) -> f64 {
        // Simple: use word probability as proxy
        // Real implementation would use sense-annotated corpus
        self.word_probability(word, context)
    }
}
```

### LLM Integration Pattern

```rust
use canopy::kernel::incremental::{SurprisalModel, Surprisal};
use std::sync::Arc;

/// Wrapper for external LLM API
pub struct LlmSurprisalModel {
    client: Arc<dyn LlmClient>,  // Your LLM client trait
    cache: parking_lot::Mutex<lru::LruCache<String, f64>>,
}

impl LlmSurprisalModel {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self {
            client,
            cache: parking_lot::Mutex::new(lru::LruCache::new(10_000)),
        }
    }

    fn cache_key(token: &str, prefix: &[&str]) -> String {
        format!("{}|||{}", prefix.join(" "), token)
    }
}

impl SurprisalModel for LlmSurprisalModel {
    fn word_probability(&self, token: &str, prefix: &[&str]) -> f64 {
        let key = Self::cache_key(token, prefix);

        // Check cache first
        if let Some(&p) = self.cache.lock().get(&key) {
            return p;
        }

        // Query LLM for log probability
        let context = prefix.join(" ");
        let log_prob = self.client
            .get_token_log_prob(&context, token)
            .unwrap_or(-20.0);  // Default for errors

        let prob = log_prob.exp();

        // Cache result
        self.cache.lock().put(key, prob);

        prob
    }

    fn sense_probability(&self, sense_id: &str, word: &str, context: &[&str]) -> f64 {
        // For sense disambiguation, you might:
        // 1. Generate sense definitions and score likelihood
        // 2. Use example sentences for each sense
        // 3. Fall back to word probability

        // Simple fallback:
        self.word_probability(word, context)
    }
}

// Example LLM client trait (implement for your API)
pub trait LlmClient: Send + Sync {
    fn get_token_log_prob(&self, context: &str, token: &str) -> Result<f64, Error>;
}
```

## Integration Points

### 1. Incremental Processing

```rust
use canopy::{IncrementalProcessor, IncrementalState, UniformSurprisalModel};
use canopy::runtime::TokenId;

let processor = IncrementalProcessor::new();
let mut state = processor.new_state();
let model = UniformSurprisalModel::default();

// Process words left-to-right
let words = ["The", "horse", "raced", "past", "the", "barn", "fell"];

for (i, word) in words.iter().enumerate() {
    let surprisal = processor.process_word(
        &mut state,
        TokenId::new(i as u32),
        word,
        &model,
    );
    println!("{}: {:.2} bits", word, surprisal.bits());
}

// Check for garden-path effects
println!("Total surprisal: {:.2} bits", state.total_surprisal().bits());
```

### 2. Discourse Context

```rust
use canopy::{DiscourseContext, DiscourseConfig, UniformSurprisalModel};

let model = UniformSurprisalModel::default();

// Attach model to discourse context
let ctx = DiscourseContext::new(DiscourseConfig::default())
    .with_surprisal_model(model);

// Now coherence classification uses surprisal adjustment
// High surprisal delta = lower coherence confidence
```

### 3. Disambiguation

```rust
use canopy::{MinSurprisalDisambiguator, Disambiguator};
use canopy::kernel::underspec::DisambiguationContext;

// Create context with model
let model = UniformSurprisalModel::default();
let ctx = DisambiguationContext::with_surprisal_model(&model);

// MinSurprisalDisambiguator selects lowest-surprisal reading
let disambiguator = MinSurprisalDisambiguator;
let best = disambiguator.select_reading(&packed_semantics, &ctx);
```

## Garden-Path Detection

Surprisal spikes indicate reanalysis points:

```rust
use canopy::{GardenPathDetector, IncrementalProcessor, UniformSurprisalModel};

let detector = GardenPathDetector::with_threshold(10.0);  // 10 bits
let processor = IncrementalProcessor::new();
let mut state = processor.new_state();
let model = UniformSurprisalModel::default();

// "The horse raced past the barn fell"
let words = ["The", "horse", "raced", "past", "the", "barn", "fell"];

for (i, word) in words.iter().enumerate() {
    let surprisal = processor.process_word(&mut state, TokenId::new(i as u32), word, &model);

    if let Some(event) = detector.check(surprisal) {
        println!("Garden-path at '{}': {:.1} bits", word, surprisal.bits());
        // "fell" likely triggers garden-path - requires reanalysis
    }
}
```

## Performance Considerations

### Caching

LLM calls are expensive. Always cache:

```rust
// Good: Cache at model level
impl SurprisalModel for MyModel {
    fn word_probability(&self, token: &str, prefix: &[&str]) -> f64 {
        let key = make_key(token, prefix);
        if let Some(p) = self.cache.get(&key) {
            return *p;
        }
        let p = self.expensive_computation(token, prefix);
        self.cache.insert(key, p);
        p
    }
}
```

### Batch Processing

For throughput, batch LLM requests:

```rust
// Collect all tokens needing probability estimates
let tokens_to_score: Vec<(String, Vec<String>)> = /* ... */;

// Single batched API call
let probs = model.batch_score(&tokens_to_score)?;

// Use results
for ((token, prefix), prob) in tokens_to_score.iter().zip(probs) {
    cache.insert(make_key(token, prefix), prob);
}
```

### Latency Targets

| Operation     | Target  | Notes           |
| ------------- | ------- | --------------- |
| Cache hit     | \<1μs   | Hash lookup     |
| N-gram lookup | \<10μs  | In-memory hash  |
| Local LLM     | \<10ms  | GPU inference   |
| API LLM       | \<100ms | Network latency |

For real-time processing, prefer local models or aggressive caching.

## Theoretical Background

### Surprisal Theory (Hale 2001)

Processing difficulty at word w is proportional to surprisal:

```
Difficulty(w) ∝ S(w) = -log₂ P(w | context)
```

### Entropy Reduction (Levy 2008)

Reading time correlates with entropy reduction—how much uncertainty decreases:

```
ΔH = H(interpretation | w₁...wₙ₋₁) - H(interpretation | w₁...wₙ)
```

### Garden-Path Theory

High surprisal (>10 bits) indicates:

1. Parser committed to wrong analysis
1. Reanalysis required
1. Processing slowdown expected

Classic example: "The horse raced past the barn fell"

- "raced" initially parsed as main verb (low surprisal)
- "fell" forces reanalysis as reduced relative (high surprisal)

## See Also

- [UNDERSPECIFICATION.md](UNDERSPECIFICATION.md) — Disambiguation strategies using surprisal
- [DISCOURSE.md](DISCOURSE.md) — Coherence adjustment with surprisal
- [FORMAL_SEMANTICS.md](FORMAL_SEMANTICS.md) — Theoretical foundations
