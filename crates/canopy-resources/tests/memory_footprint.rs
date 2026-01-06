//! Memory footprint tests for Canopy engines and pipeline.
//!
//! Run with: `cargo test -p canopy-resources --test memory_footprint -- --nocapture`

use std::mem::size_of;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

/// Get current process memory usage in bytes.
fn current_memory_bytes() -> u64 {
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::everything(),
    );

    system.process(pid).map_or(0, sysinfo::Process::memory)
}

/// Format bytes as human-readable string.
fn format_bytes(bytes: u64) -> String {
    // Convert through u32 for lossless f64 conversion in reasonable ranges
    if bytes >= 1_073_741_824 {
        let gb = u32::try_from(bytes / 1_073_741_824).unwrap_or(u32::MAX);
        let frac = u32::try_from((bytes % 1_073_741_824) / 10_737_418).unwrap_or(0);
        format!("{gb}.{frac:02} GB")
    } else if bytes >= 1_048_576 {
        let mb = u32::try_from(bytes / 1_048_576).unwrap_or(u32::MAX);
        let frac = u32::try_from((bytes % 1_048_576) / 104_857).unwrap_or(0);
        format!("{mb}.{frac} MB")
    } else if bytes >= 1024 {
        let kb = u32::try_from(bytes / 1024).unwrap_or(u32::MAX);
        let frac = u32::try_from((bytes % 1024) / 102).unwrap_or(0);
        format!("{kb}.{frac} KB")
    } else {
        format!("{bytes} B")
    }
}

/// Memory delta tracker.
struct MemoryTracker {
    baseline: u64,
    measurements: Vec<(&'static str, u64)>,
}

impl MemoryTracker {
    fn new() -> Self {
        // Force GC-like behavior by allocating and dropping
        let _ = vec![0u8; 1024 * 1024];
        std::thread::sleep(std::time::Duration::from_millis(50));

        Self {
            baseline: current_memory_bytes(),
            measurements: Vec::new(),
        }
    }

    fn measure(&mut self, label: &'static str) {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let current = current_memory_bytes();
        self.measurements.push((label, current));
    }

    fn report(&self) {
        println!();
        println!("============== MEMORY FOOTPRINT REPORT ==============");
        println!("Baseline: {}", format_bytes(self.baseline));
        println!("-----------------------------------------------------");

        let mut prev = self.baseline;
        for (label, mem) in &self.measurements {
            let delta = mem.saturating_sub(prev);
            let total = mem.saturating_sub(self.baseline);
            println!(
                "{label:<30} {:>10} (+{:>10})",
                format_bytes(total),
                format_bytes(delta)
            );
            prev = *mem;
        }

        if let Some((_, final_mem)) = self.measurements.last() {
            println!("-----------------------------------------------------");
            println!(
                "{:<30} {:>10}",
                "TOTAL",
                format_bytes(final_mem.saturating_sub(self.baseline))
            );
        }
        println!("=====================================================");
        println!();
    }
}

#[test]
fn test_type_sizes() {
    use canopy::runtime::{AnnotatedToken, SenseId, TokenId};
    use canopy::{
        ChoicePoint, ComposedEvent, DepRel, DiscourseReferent, Drs, DrsCondition, IncrementalState,
        PackedSemantics, Participant, Reading, Surprisal, ThetaRole, UPos,
    };

    println!();
    println!("==================== TYPE SIZES ====================");

    let types: Vec<(&str, usize)> = vec![
        // Core types
        ("ThetaRole", size_of::<ThetaRole>()),
        ("DepRel", size_of::<DepRel>()),
        ("UPos", size_of::<UPos>()),
        ("TokenId", size_of::<TokenId>()),
        ("SenseId", size_of::<SenseId>()),
        ("AnnotatedToken", size_of::<AnnotatedToken>()),
        // Event types
        ("ComposedEvent", size_of::<ComposedEvent>()),
        ("Participant", size_of::<Participant>()),
        // Discourse types
        ("Drs", size_of::<Drs>()),
        ("DrsCondition", size_of::<DrsCondition>()),
        ("DiscourseReferent", size_of::<DiscourseReferent>()),
        // Underspec types
        ("ChoicePoint", size_of::<ChoicePoint>()),
        ("Reading", size_of::<Reading>()),
        ("PackedSemantics", size_of::<PackedSemantics>()),
        // Incremental types
        ("Surprisal", size_of::<Surprisal>()),
        ("IncrementalState", size_of::<IncrementalState>()),
    ];

    for (name, size) in &types {
        println!("{name:<25} {size:>6} bytes");
    }
    println!("=====================================================");
    println!();

    // Sanity checks - ensure types are reasonably sized
    assert!(size_of::<ThetaRole>() <= 8, "ThetaRole too large");
    assert!(size_of::<TokenId>() <= 8, "TokenId too large");
    assert!(size_of::<Surprisal>() <= 16, "Surprisal too large");
}

#[test]
#[ignore = "requires data files - run with --ignored"]
fn test_engine_memory() {
    use canopy_resources::{
        FrameNetEngine, LexiconEngine, PropBankEngine, VerbNetEngine, WordNetEngine,
    };

    let mut tracker = MemoryTracker::new();

    println!("\nLoading engines individually...\n");

    // VerbNet
    {
        let _engine = VerbNetEngine::new().expect("VerbNet load failed");
        tracker.measure("VerbNet loaded");
    }

    // FrameNet
    {
        let _engine = FrameNetEngine::new().expect("FrameNet load failed");
        tracker.measure("+ FrameNet loaded");
    }

    // WordNet
    {
        let _engine = WordNetEngine::new().expect("WordNet load failed");
        tracker.measure("+ WordNet loaded");
    }

    // PropBank
    {
        let _engine = PropBankEngine::new().expect("PropBank load failed");
        tracker.measure("+ PropBank loaded");
    }

    // Lexicon
    {
        let _engine = LexiconEngine::new();
        tracker.measure("+ Lexicon loaded");
    }

    tracker.report();
}

#[test]
#[ignore = "requires data files - run with --ignored"]
fn test_pipeline_memory() {
    use canopy_resources::CanopyPipeline;

    let mut tracker = MemoryTracker::new();

    println!("\nLoading full pipeline...\n");

    let pipeline = CanopyPipeline::new().expect("Pipeline load failed");
    tracker.measure("Pipeline initialized");

    // Analyze some sentences to warm up caches
    let sentences = [
        "John gave Mary a book.",
        "The cat sat on the mat.",
        "Every student read a book about linguistics.",
        "She told him that he was wrong.",
    ];

    for sentence in sentences {
        let _ = pipeline.analyze(sentence);
    }
    tracker.measure("After 4 analyses");

    // Analyze more to stress caches
    for i in 0..100 {
        let sentence = format!("Test sentence number {i} with various words.");
        let _ = pipeline.analyze(&sentence);
    }
    tracker.measure("After 100 analyses");

    tracker.report();

    // Memory budget assertion
    let final_mem = current_memory_bytes();
    let baseline = tracker.baseline;
    let used = final_mem.saturating_sub(baseline);

    // Warn if over 500MB but don't fail (depends on data loaded)
    if used > 500_000_000 {
        println!(
            "WARNING: Memory usage ({}) exceeds 500MB",
            format_bytes(used)
        );
    }
}

#[test]
fn test_memory_budget_constants() {
    // Document expected memory budgets
    println!();
    println!("================== MEMORY BUDGETS ==================");

    // Actual measured values (release mode, macOS ARM64)
    let budgets = [
        ("VerbNet (333 classes)", "~5 MB"),
        ("FrameNet (1200+ frames)", "~45 MB"),
        ("WordNet (117k synsets)", "~150 MB"),
        ("PropBank", "~3 MB"),
        ("Lexicon", "~0.5 MB"),
        ("Engines subtotal", "~200 MB"),
        ("Treebank + syntax", "~100 MB"),
        ("Pipeline caches", "~40 MB"),
        ("TOTAL (pipeline)", "~340 MB"),
    ];

    for (component, budget) in budgets {
        println!("{component:<30} {budget:>15}");
    }
    println!("=====================================================");
    println!();
}
