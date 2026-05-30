# Architectural Redesign: Multi-Platform Problem Provider

## Synthesis Decision

After exploring three alternatives, the chosen design extracts a `ProblemProvider` trait to eliminate the `leetcode.rs`/`deepml.rs` duplication, splits `routine.rs` into focused modules, and introduces a typed `Schedule` model. The design prioritizes deleting duplicated code over adding abstraction layers.

## Module Map

```
src/
├── lib.rs                    (exports)
├── types.rs                  (Difficulty, Platform, Problem, ProblemResult, ProblemCache)
├── config.rs                 (unchanged)
├── utils.rs                  (unchanged)
├── api.rs                    (unchanged)
├── message.rs                (simplified: remove format_problem_message)
├── notification/             (unchanged)
├── scheduler.rs              (new: difficulty scheduling)
├── format.rs                 (new: message formatting)
├── serialization.rs          (new: JSON/XML)
├── routine.rs                (reduced: orchestration only)
├── providers/                (new: replaces leetcode.rs + deepml.rs)
│   ├── mod.rs                (ProblemProvider trait + shared selection logic)
│   ├── leetcode.rs           (LeetCode provider)
│   └── deepml.rs             (DeepML provider)
└── mcp/mod.rs                (updated for new API)
```

## Type Sketch

### ProblemProvider trait

```rust
#[async_trait::async_trait]
pub trait ProblemProvider: Send + Sync {
    fn platform(&self) -> Platform;
    async fn get_problem(
        &self,
        used_file: &str,
        difficulty: Difficulty,
    ) -> Result<ProblemResult>;
}
```

### Shared selection logic (pure function)

```rust
pub fn select_problem(
    cache_file: &str,
    used_file: &str,
    difficulty: Difficulty,
    platform: Platform,
    url_generator: impl Fn(&ProblemCache) -> String,
    seed: u64,
) -> Result<ProblemResult> {
    // Read cache, filter by difficulty, filter used, pick random
    // ...
}
```

### Schedule type

```rust
pub enum Schedule {
    Weekday { difficulty: Difficulty },
    Weekend { difficulties: [Difficulty; 2] },
}

impl Schedule {
    pub fn iter(&self) -> impl Iterator<Item = &Difficulty> {
        match self {
            Schedule::Weekday { difficulty } => std::slice::from_ref(difficulty).iter(),
            Schedule::Weekend { difficulties } => difficulties.iter(),
        }
    }
}
```

### Scheduler

```rust
pub struct Scheduler {
    week_seed: u64,
    date: DateTime<Tz>,
}

impl Scheduler {
    pub fn for_date(date: DateTime<Tz>) -> Self;
    pub fn get_schedule(&self, platform: Platform) -> Schedule;
}
```

### Section-based RoutineOptions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Problems,
    Running,
    History,
    Quote,
    YearProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineOptions {
    pub routine_type: RoutineType,
    pub sections: Vec<Section>,
    pub format: OutputFormat,
}

impl RoutineOptions {
    pub fn morning() -> Self { ... }
    pub fn problems_only() -> Self { ... }
    pub fn has_section(&self, section: Section) -> bool { ... }
}
```

## Rationale

### Why a trait + shared function instead of a generic struct?

The trait + shared function approach is cleaner because:
- The selection logic is identical (read cache, filter, pick random)
- Only URL generation and daily challenge differ
- A generic struct would require closures or trait objects for URL generation, adding indirection
- The trait keeps the provider-specific code (daily challenge, sync) in the provider file

### Why Schedule enum instead of Vec<Difficulty>?

The `Vec<Difficulty>` was a type lie. Weekdays always produce exactly 1 difficulty. The enum makes the shape explicit: `Weekday(Difficulty)` or `Weekend([Difficulty; 2])`. The `iter()` method provides a uniform interface for the caller.

### Why split routine.rs into 4 modules?

- `scheduler.rs`: ~80 lines (scheduling logic only)
- `format.rs`: ~60 lines (message formatting)
- `serialization.rs`: ~120 lines (JSON + XML with helper functions)
- `routine.rs`: ~150 lines (orchestration only)

This keeps each file under 300 lines and gives each module a single responsibility.

### Why Section enum instead of 5 booleans?

The booleans forced the caller to check 5 independent flags. The enum makes the configuration a set, which is the natural model. The MCP API can still accept the old booleans and map them to sections internally.

## Migration Path

1. Create `providers/mod.rs` with shared selection logic
2. Refactor `leetcode.rs` and `deepml.rs` into `providers/leetcode.rs` and `providers/deepml.rs`
3. Create `scheduler.rs` with `Schedule` enum
4. Create `format.rs` with `build_formatted_message`
5. Create `serialization.rs` with `to_json` and `to_xml`
6. Reduce `routine.rs` to orchestration
7. Update `main.rs` and `mcp/mod.rs` to use new API
8. Delete old `leetcode.rs` and `deepml.rs` from root

## Backward Compatibility

The MCP API surface (`RunRoutineParams`) keeps the same JSON fields (`include_problems`, `include_running`, etc.) but maps them to `Vec<Section>` internally. No external clients need to change.

## File Size Targets

| File | Current | Target |
|------|---------|--------|
| routine.rs | 777 | ~150 |
| scheduler.rs | 0 | ~80 |
| format.rs | 0 | ~60 |
| serialization.rs | 0 | ~120 |
| providers/mod.rs | 0 | ~100 |
| providers/leetcode.rs | 0 | ~120 |
| providers/deepml.rs | 0 | ~120 |
| message.rs | 193 | ~80 |
