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
mod proof;
mod query;
mod reasoner;
mod solver;

pub use answer::{Answer, AnswerBinding, QueryResult};
pub use compiler::{compile, CompiledDrs, Fact, NegatedFormula};
pub use proof::{ConditionRef, Explanation, ExplanationStep, StepKind};
pub use query::{qud_to_query, Constraint, Proposition, Query, Term};
pub use reasoner::{Conflict, ConsistencyResult, Entailment, EntailmentResult, Reasoner};
pub use solver::ClosedWorldReasoner;
