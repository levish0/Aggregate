use aggregate_programs::{InspectionScope, InspectionSection};
use aggregate_simulation_core::{
    CommandOutcome, DayReport, Simulation, SimulationCommand, SimulationError,
};
use aggregate_world::WorldSnapshotIndex;
use aggregate_world::{FacilityDefinitionId, FacilityId, WorldSnapshot};
use std::{
    sync::{Mutex, mpsc},
    time::Instant,
};

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
    pub construction_id: Option<FacilityId>,
    pub result: Result<WorkOutcome, SimulationError>,
    pub snapshot: Option<(WorldSnapshot, WorldSnapshotIndex)>,
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
        std::thread::Builder::new()
            .name("aggregate-simulation".into())
            .spawn(move || {
                while let Ok(first) = incoming.recv() {
                    let started = Instant::now();
                    let mut responses = Vec::new();
                    let mut changed = false;
                    let requests = std::iter::once(first).chain(incoming.try_iter().take(31));
                    for request in requests {
                        let construction_id = match &request {
                            WorkRequest::Construction(SimulationCommand::StartConstruction { facility, .. }, _) => Some(facility.clone()),
                            _ => None,
                        };
                        let result = match request {
                            WorkRequest::Step => simulation.step().map(WorkOutcome::Day),
                            WorkRequest::Construction(command, definition) => simulation.execute(command).map(|outcome| WorkOutcome::Construction(definition, outcome)),
                            WorkRequest::Inspect(scope, revision) => Ok(WorkOutcome::Inspection(scope.clone(), revision, simulation.inspect_programs(&scope).map_err(|error| error.to_string()))),
                        };
                        changed |= matches!(&result, Ok(WorkOutcome::Day(_) | WorkOutcome::Construction(..)));
                        responses.push(CompletedWork { construction_id, result, snapshot: None });
                    }
                    let calculation_ms = started.elapsed().as_secs_f64() * 1000.;
                    let snapshot_started = Instant::now();
                    if changed {
                        let snapshot = simulation.snapshot();
                        let index = WorldSnapshotIndex::build(&snapshot);
                        // Publish once per drained command batch, before the UI observes outcomes.
                        responses[0].snapshot = Some((snapshot, index));
                    }
                    tracing::debug!(target: "aggregate_client::simulation_performance", requests = responses.len(), day = simulation.clock().day(), calculation_ms, snapshot_ms = snapshot_started.elapsed().as_secs_f64() * 1000., "Simulation worker completed batch");
                    if responses.into_iter().any(|response| outgoing.send(response).is_err()) { break; }
                }
            })
            .map_err(|error| {
                SimulationError::InvalidPrograms(format!("simulation worker: {error}"))
            })?;
        Ok(Self {
            requests,
            responses: Mutex::new(responses),
        })
    }

    pub fn submit(&self, request: WorkRequest) -> Result<(), SimulationError> {
        self.requests.try_send(request).map_err(|error| {
            SimulationError::CommandRejected(match error {
                mpsc::TrySendError::Full(_) => "simulation request queue is full".into(),
                mpsc::TrySendError::Disconnected(_) => "simulation worker stopped".into(),
            })
        })
    }

    pub fn try_receive(&self) -> Option<CompletedWork> {
        match self.responses.lock().expect("response receiver").try_recv() {
            Ok(completed) => Some(completed),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(CompletedWork {
                construction_id: None,
                result: Err(SimulationError::InvalidPrograms(
                    "simulation worker stopped".into(),
                )),
                snapshot: None,
            }),
        }
    }
}
