//! Planned-only scenario runner.

mod error;

pub use error::RunnerResult;

use duan_package::{AssemblyPlan, Registry};
use duan_scenario::{Manifest, RunOptions};

#[derive(Clone, Debug)]
pub struct Runner {
    pub registry: Registry,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunReport {
    pub scenario_id: String,
    pub planned_steps: usize,
    pub executed_steps: u64,
    pub simulated_time: f64,
    pub entities_planned: usize,
    pub status: RunStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunStatus {
    PlannedOnly,
}

impl Runner {
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }

    pub fn plan(&self, manifest: &Manifest) -> RunnerResult<AssemblyPlan> {
        AssemblyPlan::from_manifest(manifest, &self.registry)
    }

    pub fn run(&self, manifest: &Manifest) -> RunnerResult<RunReport> {
        let plan = self.plan(manifest)?;
        let executed_steps = executed_steps(manifest.run.as_ref());

        Ok(RunReport {
            scenario_id: manifest.scenario.id.clone(),
            planned_steps: plan.steps().len(),
            executed_steps,
            simulated_time: simulated_time(manifest.run.as_ref(), executed_steps),
            entities_planned: manifest.entities.len(),
            status: RunStatus::PlannedOnly,
        })
    }
}

pub fn run_scenario(scenario: &Manifest, registry: Registry) -> RunnerResult<RunReport> {
    Runner::new(registry).run(scenario)
}

fn executed_steps(run: Option<&RunOptions>) -> u64 {
    let Some(run) = run else {
        return 0;
    };
    if let Some(steps) = run.steps {
        return steps;
    }

    match (run.max_time, run.delta_time) {
        (Some(max_time), Some(delta_time)) if delta_time > 0.0 => {
            (max_time / delta_time).ceil().max(0.0) as u64
        }
        _ => 0,
    }
}

fn simulated_time(run: Option<&RunOptions>, executed_steps: u64) -> f64 {
    let Some(run) = run else {
        return 0.0;
    };

    if let Some(delta_time) = run.delta_time {
        executed_steps as f64 * delta_time
    } else {
        run.max_time.unwrap_or(0.0)
    }
}
