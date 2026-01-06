//! Executable logic layer for DRS inference and query answering.
//!
//! This module provides reasoning capabilities over Discourse Representation
//! Structures (DRS), enabling:
//!
//! - **Consistency checking**: Detect contradictions in discourse
//! - **Entailment**: Check if propositions follow from the discourse
//! - **Query answering**: Answer yes/no and wh-questions
//! - **Explanations**: Generate proofs with sentence-level provenance
//!
//! # Architecture
//!
//! ```text
//! DRS ──► Compiler ──► CompiledDrs ──► Reasoner ──► QueryResult
//!                                          │
//!                                          ▼
//!                                    Explanation
//! ```
//!
//! # Example
//!
//! ```ignore
//! use canopy::kernel::logic::{ClosedWorldReasoner, Query, Reasoner};
//!
//! let reasoner = ClosedWorldReasoner::new();
//! let result = reasoner.answer(&drs, &Query::yes_no("leave", "John", ThetaRole::Agent));
//! ```

mod answer;
mod compiler;
mod modal_reasoner;
mod proof;
mod query;
mod reasoner;
mod solver;
mod temporal_reasoner;

pub use answer::{Answer, AnswerBinding, QueryResult};
pub use compiler::{compile, CompiledDrs, Fact, NegatedFormula};
pub use modal_reasoner::{
    CounterfactualEvaluation, CounterfactualModal, ModalEvaluation, ModalReasoner, World,
};
pub use proof::{ConditionRef, Explanation, ExplanationStep, StepKind};
pub use query::{qud_to_query, Constraint, Proposition, Query, Term};
pub use reasoner::{Conflict, ConsistencyResult, Entailment, EntailmentResult, Reasoner};
pub use solver::ClosedWorldReasoner;
pub use temporal_reasoner::{
    AllenRelation, TemporalConsistencyResult, TemporalConstraint, TemporalReasoner,
};
