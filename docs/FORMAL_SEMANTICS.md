# Formal Semantics Specification

This appendix provides the formal semantic foundations underlying Canopy's analysis. It defines the meaning language, composition rules, and theoretical frameworks.

## Event Semantics

Canopy uses Neo-Davidsonian event semantics (Parsons 1990) where events are first-class entities and thematic roles are separate predicates.

### Event Variables

For a sentence like "John broke the vase", the logical form is:

```
∃e[Break(e) ∧ Agent(e, john) ∧ Patient(e, vase) ∧ Past(e)]
```

Where:

- `e` is an event variable
- `Break(e)` is the event predicate
- `Agent(e, john)` binds the agent role
- `Patient(e, vase)` binds the patient role
- `Past(e)` is temporal modification

### Advantages of Neo-Davidsonian Representation

1. **Modifier attachment**: Adverbials modify events directly

   ```
   "John broke the vase quickly"
   ∃e[Break(e) ∧ Agent(e, john) ∧ Patient(e, vase) ∧ Quick(e)]
   ```

1. **Optional arguments**: Roles can be omitted without changing arity

   ```
   "The vase broke"
   ∃e[Break(e) ∧ Patient(e, vase)]  // No agent required
   ```

1. **Event anaphora**: Events can be referenced

   ```
   "John broke the vase. It happened yesterday."
   ∃e[Break(e) ∧ ...] ∧ Yesterday(e)
   ```

## LittleV Decomposition

Following Hale & Keyser (1993) and Ramchand (2008), Canopy decomposes verbs into primitive event structures.

### Primitives

| Primitive  | Semantics          | Structure                                         |
| ---------- | ------------------ | ------------------------------------------------- |
| CAUSE      | External causation | `∃e∃e'[Cause(e,e') ∧ Agent(e,x) ∧ P(e')]`         |
| BECOME     | Change of state    | `∃e[Become(e) ∧ Theme(e,x) ∧ ResultState(e,s)]`   |
| BE         | Stative            | `∃s[State(s) ∧ Theme(s,x) ∧ Property(s,p)]`       |
| DO         | Activity           | `∃e[Activity(e) ∧ Agent(e,x)]`                    |
| GO         | Motion/Path        | `∃e[Motion(e) ∧ Theme(e,x) ∧ Path(e,p)]`          |
| HAVE       | Possession         | `∃s[Poss(s) ∧ Holder(s,x) ∧ Possessee(s,y)]`      |
| EXPERIENCE | Psych state        | `∃s[Psych(s) ∧ Experiencer(s,x) ∧ Stimulus(s,y)]` |

### Decomposition Examples

**Causative-inchoative alternation**:

```
"John broke the vase"     (transitive)
CAUSE(john, BECOME(vase, broken))
∃e∃e'[Cause(e,e') ∧ Agent(e,john) ∧ Become(e') ∧ Theme(e',vase) ∧ Broken(e')]

"The vase broke"          (inchoative)
BECOME(vase, broken)
∃e[Become(e) ∧ Theme(e,vase) ∧ Broken(e)]
```

**Motion**:

```
"John ran to the store"
DO(john, run) & GO(john, to(store))
∃e[Activity(e) ∧ Agent(e,john) ∧ Run(e) ∧ Goal(e,store)]
```

### VerbNet to LittleV Mapping

Canopy maps VerbNet classes to LittleV primitives:

| VerbNet Class | LittleV      | Example            |
| ------------- | ------------ | ------------------ |
| break-45.1    | CAUSE+BECOME | "break", "shatter" |
| run-51.3.2    | DO+GO        | "run", "walk"      |
| admire-31.2   | EXPERIENCE   | "admire", "fear"   |
| give-13.1     | CAUSE+HAVE   | "give", "hand"     |
| put-9.1       | CAUSE+BE     | "put", "place"     |

## Thematic Roles

### Role Hierarchy (UTAH)

The Uniformity of Theta Assignment Hypothesis (Baker 1988) maps syntactic positions to roles:

```
[Spec,vP] → Agent/Causer/Experiencer
[Spec,VP] → Theme/Patient
[Compl,V] → Theme/Goal/Recipient
[Adjunct]  → Instrument/Location/Manner
```

### Role Definitions

| Role            | Definition                 | Diagnostic                   |
| --------------- | -------------------------- | ---------------------------- |
| **Agent**       | Volitional initiator       | "deliberately", "on purpose" |
| **Patient**     | Affected, undergoes change | "What happened to X?"        |
| **Theme**       | Moved or located entity    | "Where is X?"                |
| **Experiencer** | Sentient, has mental state | Psych verbs                  |
| **Recipient**   | End-point of transfer      | Ditransitive "to X"          |
| **Goal**        | End-point of motion        | "to X", "into X"             |
| **Source**      | Starting point             | "from X"                     |
| **Instrument**  | Means of action            | "with X", "using X"          |
| **Location**    | Place                      | "at X", "in X"               |

### Proto-Role Theory

Following Dowty (1991), Canopy uses proto-role entailments:

**Proto-Agent entailments**:

- Volitional involvement
- Sentience/perception
- Causes event or change
- Movement relative to other participant

**Proto-Patient entailments**:

- Undergoes change of state
- Incremental theme
- Causally affected
- Stationary relative to other participant

## Discourse Representation Theory

Canopy implements DRT (Kamp 1981, Kamp & Reyle 1993) for discourse-level semantics.

### DRS Structure

A Discourse Representation Structure (DRS) is a pair ⟨U, Con⟩:

- U: set of discourse referents
- Con: set of conditions

```
[x, y, e |
  John(x),
  Mary(y),
  give(e),
  Agent(e,x),
  Recipient(e,y),
  Theme(e,z),
  book(z)]
```

### DRS Construction Rules

**Indefinite NP**: Introduces new referent

```
"A man entered"
[x, e | man(x), enter(e), Agent(e,x)]
```

**Definite NP**: Presupposes accessible referent

```
"The man sat down"
Requires: x accessible where man(x)
[e | sit(e), Agent(e,x)]
```

**Pronoun**: Anaphorically bound

```
"He smiled"
Requires: x accessible, suitable antecedent
[e | smile(e), Agent(e,x)]
```

### Accessibility

Referent x is accessible from DRS K if:

1. x ∈ U(K), or
1. x is accessible from a superordinate DRS

Inaccessible contexts:

- Negation: `¬[x | ...]` — x not accessible outside
- Conditional antecedent: `[x | ...] → [...]` — x not accessible in consequent to main DRS
- Disjunction: `[x | ...] ∨ [...]` — x not accessible across disjuncts

### DRS Merge

Sequential sentences merge:

```
⟦S₁. S₂⟧ = merge(⟦S₁⟧, ⟦S₂⟧)

merge(⟨U₁, Con₁⟩, ⟨U₂, Con₂⟩) = ⟨U₁ ∪ U₂, Con₁ ∪ Con₂⟩
```

## Coherence Relations (SDRT)

Segmented DRT (Asher & Lascarides 2003) adds rhetorical relations between discourse segments.

### Relation Types

| Relation        | Definition           | Cue Words                       |
| --------------- | -------------------- | ------------------------------- |
| **Narration**   | τ(e₁) < τ(e₂)        | "then", "next"                  |
| **Elaboration** | e₂ is part of e₁     | "specifically", "in particular" |
| **Explanation** | e₂ causes e₁         | "because", "since"              |
| **Result**      | e₁ causes e₂         | "so", "therefore"               |
| **Contrast**    | ¬compatible(e₁, e₂)  | "but", "however"                |
| **Parallel**    | similar(e₁, e₂)      | "similarly", "also"             |
| **Background**  | e₂ sets scene for e₁ | past progressive                |

### Formal Definition

R(α, β) where:

- α, β are discourse segment labels
- R is a coherence relation
- Constraints propagate through the structure

```
Narration(α, β): τ(main-event(α)) < τ(main-event(β))
Explanation(α, β): cause(main-event(β), main-event(α))
```

## Scope Underspecification (MRS)

Minimal Recursion Semantics (Copestake et al. 2005) represents scope ambiguity.

### MRS Structure

An MRS is a tuple ⟨GT, R, C⟩:

- GT: top handle (sentence scope)
- R: bag of elementary predications (EPs)
- C: handle constraints

### Elementary Predications

```
EP = ⟨h, p, a₁, ..., aₙ⟩
```

Where:

- h: label (handle)
- p: predicate
- aᵢ: arguments (may include handles for scope)

### Handle Constraints

**Qeq** (equality modulo quantifiers):

```
h₁ =q h₂
```

Means h₁ either equals h₂ or immediately outscopes it.

### Example: Scope Ambiguity

"Every student read a book"

```
EPs:
  h₁: every(x, h₂, h₃)
  h₄: student(x)
  h₅: a(y, h₆, h₇)
  h₈: book(y)
  h₉: read(e, x, y)

Constraints:
  h₂ =q h₄
  h₆ =q h₈

Valid scope orderings:
  ∀ > ∃: h₃ = h₅, h₇ = h₉  →  ∀x[student(x) → ∃y[book(y) ∧ read(x,y)]]
  ∃ > ∀: h₇ = h₁, h₃ = h₉  →  ∃y[book(y) ∧ ∀x[student(x) → read(x,y)]]
```

## Questions Under Discussion (QUD)

Following Roberts (1996, 2012), Canopy models discourse as organized around questions.

### Question Semantics

A question denotes a set of propositions (its possible answers):

```
⟦Who left?⟧ = {p : ∃x[p = left(x)]}
           = {left(john), left(mary), left(bill), ...}
```

### QUD Stack

Discourse maintains a stack of open questions:

```
Initial: [What is the way things are?]  (Big Question)
After Q: [Q, What is the way things are?]
After A: pop if A completely answers top Q
```

### Congruence

An assertion A is **congruent** to question Q iff A ∈ ⟦Q⟧:

```
Q: "Who left?"
A: "John left"  ✓ congruent (left(john) ∈ ⟦Who left?⟧)
A: "It's raining"  ✗ not congruent
```

### Focus-Sensitivity

Focus marks the "question-answering" part:

```
Q: "Who left?"
A: "JOHN left"     Focus on "John" — answers Q
A: "John LEFT"     Focus on "left" — answers "What did John do?"
```

## Information Structure

### Topic-Focus Articulation

```
Sentence = Topic + Focus
Topic: What the sentence is about (given)
Focus: New information (answers implicit Q)
```

### Givenness

| Status     | Definition       | Prosody        |
| ---------- | ---------------- | -------------- |
| Given      | In common ground | Deaccented     |
| Accessible | Inferable        | Reduced accent |
| New        | Not in CG        | Full accent    |

## Presupposition

### Triggers

| Trigger         | Example           | Presupposition  |
| --------------- | ----------------- | --------------- |
| Definite        | "the king"        | ∃x[king(x)]     |
| Factive         | "know that p"     | p               |
| Change of state | "stop V-ing"      | was V-ing       |
| Cleft           | "It was X who..." | someone did...  |
| Iterative       | "again", "return" | happened before |

### Accommodation

When presupposition P is not in common ground CG:

1. **Global accommodation**: Add P to CG
1. **Local accommodation**: Add P to embedded DRS
1. **Failure**: Reject as infelicitous

## References

- Asher, N., & Lascarides, A. (2003). *Logics of Conversation*. Cambridge.
- Baker, M. (1988). *Incorporation*. Chicago.
- Copestake, A., et al. (2005). Minimal Recursion Semantics. *Research on Language and Computation*.
- Dowty, D. (1991). Thematic proto-roles and argument selection. *Language*.
- Hale, J. (2001). A probabilistic Earley parser as a psycholinguistic model. *NAACL*.
- Hale, K., & Keyser, S.J. (1993). On argument structure. *The View from Building 20*.
- Kamp, H. (1981). A theory of truth and semantic representation. *Formal Methods*.
- Kamp, H., & Reyle, U. (1993). *From Discourse to Logic*. Kluwer.
- Levy, R. (2008). Expectation-based syntactic comprehension. *Cognition*.
- Parsons, T. (1990). *Events in the Semantics of English*. MIT Press.
- Ramchand, G. (2008). *Verb Meaning and the Lexicon*. Cambridge.
- Roberts, C. (1996/2012). Information structure in discourse. *Semantics & Pragmatics*.

## See Also

- [DISCOURSE.md](DISCOURSE.md) — Implementation of DRT and coherence
- [UNDERSPECIFICATION.md](UNDERSPECIFICATION.md) — MRS-style scope handling
- [SURPRISAL.md](SURPRISAL.md) — Information-theoretic processing
