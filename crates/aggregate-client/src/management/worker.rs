use super::snapshot_index::SnapshotIndex;
use aggregate_programs::{InspectionScope, InspectionSection};
use aggregate_simulation_core::{CommandOutcome, DayReport, Simulation, SimulationCommand, SimulationError};
use aggregate_world::{FacilityDefinitionId, WorldSnapshot};
use std::{sync::{Mutex, mpsc}, time::Instant};

pub(super) enum WorkRequest {
    Step,
    Construction(SimulationCommand, FacilityDefinitionId),
    Inspect(InspectionScope, u64),
}

pub(super) enum WorkOutcome {
    Day(DayReport),
    Construction(FacilityDefinitionId, CommandOutcome),
    Inspection(InspectionScope, u64, Result<Vec<InspectionSection>, String>),
}

pub(super) struct CompletedWork {
    pub result: Result<WorkOutcome, SimulationError>,
    pub snapshot: Option<(WorldSnapshot, SnapshotIndex)>,
}

/// A dedicated thread owns the sole Simulation. Requests are serialized between
/// complete ticks; UI submission and result polling never wait for that thread.
pub(super) struct SimulationWorker {
    requests: mpsc::SyncSender<WorkRequest>,
    responses: Mutex<mpsc::Receiver<CompletedWork>>,
}

impl SimulationWorker {
    pub fn spawn(mut simulation: Simulation) -> Result<Self, SimulationError> {
        let (requests, incoming) = mpsc::sync_channel::<WorkRequest>(32);
        let (outgoing, responses) = mpsc::channel();
        std::thread::Builder::new().name("aggregate-simulation".into()).spawn(move || {
            while let Ok(request) = incoming.recv() {
                let started = Instant::now();
                let result = match request {
                    WorkRequest::Step => simulation.step().map(WorkOutcome::Day),
                    WorkRequest::Construction(command, definition) => simulation.execute(command).map(|outcome| WorkOutcome::Construction(definition, outcome)),
                    WorkRequest::Inspect(scope, revision) => {
                        let result = simulation.inspect_programs(&scope).map_err(|error| error.to_string());
                        Ok(WorkOutcome::Inspection(scope, revision, result))
                    }
                };
                let calculation_ms = started.elapsed().as_secs_f64() * 1000.;
                let snapshot = if matches!(&result, Ok(WorkOutcome::Day(_) | WorkOutcome::Construction(..))) {
                    let snapshot = simulation.snapshot();
                    let index = SnapshotIndex::build(&snapshot);
                    Some((snapshot, index))
                } else { None };
                tracing::debug!(target: "aggregate_client::simulation_performance", day = simulation.clock().day(), calculation_ms, snapshot_ms = started.elapsed().as_secs_f64() * 1000. - calculation_ms, "Simulation worker completed request");
                if outgoing.send(CompletedWork { result, snapshot }).is_err() { break; }
            }
        }).map_err(|error| SimulationError::InvalidPrograms(format!("simulation worker: {error}")))?;
        Ok(Self { requests, responses: Mutex::new(responses) })
    }

    pub fn submit(&self, request: WorkRequest) -> Result<(), SimulationError> {
        self.requests.try_send(request).map_err(|error| SimulationError::CommandRejected(match error {
            mpsc::TrySendError::Full(_) => "simulation request queue is full".into(),
            mpsc::TrySendError::Disconnected(_) => "simulation worker stopped".into(),
        }))
    }

    pub fn try_receive(&self) -> Option<CompletedWork> {
        match self.responses.lock().expect("response receiver").try_recv() {
            Ok(completed) => Some(completed),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(CompletedWork {
                result: Err(SimulationError::InvalidPrograms("simulation worker stopped".into())), snapshot: None,
            }),
        }
    }
}
