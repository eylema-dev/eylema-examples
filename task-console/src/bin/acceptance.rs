use eylema_contracts::{
    codec::{self, Value},
    transport::Request,
    *,
};
use eylema_task_console_example::{
    Client, ClientError, ExecutionSnapshot, prepare_demo, prepare_request_payload,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CORE_REVISION: &str = "b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f";

fn main() {
    if let Err(error) = run() {
        eprintln!("G17 acceptance failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let core_root = std::env::args()
        .nth(1)
        .ok_or("usage: acceptance <pinned-core-checkout>")?;
    let revision = Command::new("git")
        .args(["-C", &core_root, "rev-parse", "HEAD"])
        .output()
        .map_err(|_| "could not inspect the core checkout")?;
    if !revision.status.success()
        || String::from_utf8_lossy(&revision.stdout).trim() != CORE_REVISION
    {
        return Err(format!("core checkout must be exactly {CORE_REVISION}"));
    }
    let binary = Path::new(&core_root).join("target/debug/eylema");
    if !binary.is_file() {
        return Err("build the pinned core CLI first".into());
    }

    let root = TempRoot::create("journey")?;
    let mut service = Service::start(&binary, root.path.clone())?;
    let client = Client::from_data_dir(&root.path).map_err(|error| error.to_string())?;

    // F1: create and inspect a valid task; reject invalid input locally.
    if prepare_demo(0, "task:state").is_ok() {
        return Err("F1 accepted a zero-sized demo input".into());
    }
    let (task, admission) = prepare_demo(7, "task:state").map_err(|e| e.to_string())?;
    let task_id = task.fields().task_id.clone();
    client
        .request(Request::Create(task.clone()))
        .map_err(|e| format!("F1 create: {e}"))?;
    let created = client
        .inspect_task(&task_id)
        .map_err(|e| format!("F1 inspect: {e}"))?;
    if created.value != 0 || created.task_id != task_id.as_str() {
        return Err("F1 returned an unexpected initial task snapshot".into());
    }
    println!("F1 passed: {}", created.task_id);

    // F2: run one deterministic, authorized step and inspect its execution.
    let execution_id = admit(&client, admission)?;
    let finished = wait_terminal(&client, &execution_id)?;
    let completed = client
        .inspect_task(&task_id)
        .map_err(|e| format!("F2 inspect: {e}"))?;
    if completed.value != 7
        || completed.performed_steps != 1
        || finished.state != ExecutionState::Succeeded
    {
        return Err("F2 did not produce the expected ordered terminal result".into());
    }
    println!("F2 passed: value=7, steps=1, state={:?}", finished.state);

    // F3: the public stock session is non-human; denial must preserve terminal state.
    let before = client
        .inspect_task(&task_id)
        .map_err(|e| format!("F3 pre-inspect: {e}"))?;
    match client.cancel(execution_id.clone()) {
        Err(ClientError::Remote(code)) if code == "FORBIDDEN" => {}
        Err(error) => return Err(format!("F3 returned an unexpected stop result: {error}")),
        Ok(_) => {
            return Err("F3 elevated the non-human session into cancellation authority".into());
        }
    }
    let after = client
        .inspect_task(&task_id)
        .map_err(|e| format!("F3 post-inspect: {e}"))?;
    let stopped = client
        .inspect_execution(&execution_id)
        .map_err(|e| format!("F3 execution: {e}"))?;
    if before != after || stopped.state != ExecutionState::Succeeded {
        return Err("F3 denial changed the completed task".into());
    }
    println!("F3 passed: FORBIDDEN left the completed result unchanged");

    // Record an event page so F4 checks the same durable observations after restart.
    let events_before = client
        .request(Request::Events {
            task: task_id.clone(),
            after: None,
            limit: 256,
        })
        .map_err(|e| format!("F4 events before restart: {e}"))?;
    service.stop()?;
    service = Service::start(&binary, root.path.clone())?;
    let restarted = Client::from_data_dir(&root.path).map_err(|e| e.to_string())?;
    let recovered_task = restarted
        .inspect_task(&task_id)
        .map_err(|e| format!("F4 task recovery: {e}"))?;
    let recovered_execution = restarted
        .inspect_execution(&execution_id)
        .map_err(|e| format!("F4 execution recovery: {e}"))?;
    let events_after = restarted
        .request(Request::Events {
            task: task_id.clone(),
            after: None,
            limit: 256,
        })
        .map_err(|e| format!("F4 events after restart: {e}"))?;
    if recovered_task != completed
        || recovered_execution != finished
        || events_before != events_after
    {
        return Err("F4 changed task/execution identity or events across service restart".into());
    }
    println!("F4 passed: task, execution and event page recovered after service restart");

    // F5: identical admission replay keeps its execution identity; changed payload
    // under the same idempotency key and a stale revision fail explicitly.
    let (replay_task, replay_request) = prepare_demo(3, "task:state").map_err(|e| e.to_string())?;
    restarted
        .request(Request::Create(replay_task.clone()))
        .map_err(|e| format!("F5 create: {e}"))?;
    let first = admit(&restarted, replay_request.clone())?;
    let replayed = admit(&restarted, replay_request.clone())?;
    if first != replayed {
        return Err("F5 duplicate request did not return the original execution".into());
    }
    let _ = wait_terminal(&restarted, &first)?;
    let changed = reissue(
        &replay_request,
        "changed",
        Some(codec::encode(&Value::Unsigned(8)).map_err(|_| "encode failed")?),
    )?;
    expect_remote(
        &restarted,
        changed,
        "IDEMPOTENCY_CONFLICT",
        "F5 changed payload",
    )?;
    let stale = reissue(&replay_request, "stale", None)?;
    expect_remote(&restarted, stale, "REVISION_CONFLICT", "F5 stale revision")?;
    println!("F5 passed: identical replay stable; changed payload and stale revision rejected");

    // F6: an incompatible private journal fails closed on a disposable profile.
    service.stop()?;
    let corrupt_root = TempRoot::create("incompatible-journal")?;
    let mut corrupt_service = Service::start(&binary, corrupt_root.path.clone())?;
    let corrupt_client = Client::from_data_dir(&corrupt_root.path).map_err(|e| e.to_string())?;
    let (corrupt_task, _) = prepare_demo(1, "task:state").map_err(|e| e.to_string())?;
    corrupt_client
        .request(Request::Create(corrupt_task))
        .map_err(|e| format!("F6 seed: {e}"))?;
    corrupt_service.stop()?;
    fs::write(
        corrupt_root.path.join("journal.eylj"),
        b"incompatible fixture",
    )
    .map_err(|_| "could not write disposable incompatible journal")?;
    set_private_file(&corrupt_root.path.join("journal.eylj"));
    let rejected = Command::new(&binary)
        .arg("start")
        .env("EYLEMA_DATA_DIR", &corrupt_root.path)
        .output()
        .map_err(|_| "could not restart with incompatible fixture")?;
    let error_text = String::from_utf8_lossy(&rejected.stdout);
    if rejected.status.success() || !error_text.contains("JOURNAL_INVALID") {
        return Err("F6 did not fail closed on the incompatible journal".into());
    }
    println!("F6 passed: incompatible journal returned JOURNAL_INVALID without recovery claims");

    // F7: unavailable capabilities and oversized requests remain explicit/bounded.
    service = Service::start(&binary, root.path.clone())?;
    let f7_client = Client::from_data_dir(&root.path).map_err(|e| e.to_string())?;
    let (unavailable_task, unavailable_request) =
        prepare_demo(1, "capability:g17-unavailable").map_err(|e| e.to_string())?;
    f7_client
        .request(Request::Create(unavailable_task))
        .map_err(|e| format!("F7 create: {e}"))?;
    expect_remote(
        &f7_client,
        unavailable_request,
        "CAPABILITY_UNAVAILABLE",
        "F7 unavailable capability",
    )?;
    let (_, oversized) =
        prepare_request_payload("task:state", vec![0; 66 * 1024]).map_err(|e| e.to_string())?;
    if f7_client.request(oversized) != Err(ClientError::Remote("REQUEST_TOO_LARGE".into())) {
        return Err("F7 did not reject an oversized request at the consumer boundary".into());
    }
    println!("F7 passed: unavailable capability and oversized request failed explicitly");

    service.stop()?;
    println!("PASS: all seven G17 scenarios on core {CORE_REVISION}");
    Ok(())
}

fn admit(client: &Client, request: Request) -> Result<ExecutionId, String> {
    match client.request(request).map_err(|e| e.to_string())? {
        Value::Text(id) => ExecutionId::new(id).map_err(|_| "malformed execution ID".into()),
        _ => Err("admission returned a malformed response".into()),
    }
}

fn wait_terminal(client: &Client, id: &ExecutionId) -> Result<ExecutionSnapshot, String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = client.inspect_execution(id).map_err(|e| e.to_string())?;
        if snapshot.state.is_terminal() {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err("execution outcome remained unknown after five seconds".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn expect_remote(
    client: &Client,
    request: Request,
    expected: &str,
    case: &str,
) -> Result<(), String> {
    match client.request(request) {
        Err(ClientError::Remote(code)) if code == expected => Ok(()),
        Err(error) => Err(format!("{case}: expected {expected}, received {error}")),
        Ok(_) => Err(format!("{case}: expected {expected}, request succeeded")),
    }
}

fn reissue(request: &Request, suffix: &str, payload: Option<Vec<u8>>) -> Result<Request, String> {
    let Request::Admit {
        task,
        proposal,
        approvals,
    } = request
    else {
        return Err("expected an admission request".into());
    };
    let mut draft = proposal.fields().clone();
    draft.proposal_id =
        ProposalId::new(format!("proposal:g17-{suffix}")).map_err(|e| e.to_string())?;
    let action = draft
        .actions
        .first()
        .ok_or("missing action")?
        .fields()
        .clone();
    let mut changed = action;
    changed.action_id = ActionId::new(format!("action:g17-{suffix}")).map_err(|e| e.to_string())?;
    if let Some(payload) = payload {
        changed.payload = payload;
    } else {
        changed.idempotency_key =
            IdempotencyKey::new(format!("g17-{suffix}:step:1")).map_err(|e| e.to_string())?;
    }
    draft.actions = vec![ActionSpec::new(changed).map_err(|e| e.to_string())?];
    let proposal = ActionProposal::new(draft, task).map_err(|e| e.to_string())?;
    Ok(Request::Admit {
        task: task.clone(),
        proposal,
        approvals: approvals.clone(),
    })
}

struct Service {
    binary: PathBuf,
    data_dir: PathBuf,
    child: Option<Child>,
}

impl Service {
    fn start(binary: &Path, data_dir: PathBuf) -> Result<Self, String> {
        let mut child = Command::new(binary)
            .arg("start")
            .env("EYLEMA_DATA_DIR", &data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "could not start the pinned core service")?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some(status) = child
                .try_wait()
                .map_err(|_| "could not inspect core service")?
            {
                return Err(format!("core service exited during start: {status}"));
            }
            if Client::from_data_dir(&data_dir).is_ok() {
                return Ok(Self {
                    binary: binary.to_owned(),
                    data_dir,
                    child: Some(child),
                });
            }
            thread::sleep(Duration::from_millis(20));
        }
        let _ = child.kill();
        Err("core service did not create a private local session in time".into())
    }

    fn stop(&mut self) -> Result<(), String> {
        let Some(child) = self.child.as_mut() else {
            return Ok(());
        };
        let request = Command::new(&self.binary)
            .args(["stop", "service"])
            .env("EYLEMA_DATA_DIR", &self.data_dir)
            .output()
            .map_err(|_| "could not request graceful core-service stop")?;
        if !request.status.success() {
            return Err("core CLI rejected graceful service stop".into());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|_| "could not inspect stopped service")?
            {
                if !status.success() {
                    return Err(format!("core service stopped with {status}"));
                }
                self.child = None;
                return Ok(());
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                return Err("core service did not stop gracefully within five seconds".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if self.child.is_some() {
            let _ = self.stop();
        }
    }
}

struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn create(label: &str) -> Result<Self, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "clock unavailable")?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("eylema-g17-{label}-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).map_err(|_| "could not create private disposable test directory")?;
        set_private_dir(&path);
        Ok(Self { path })
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(unix)]
fn set_private_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn set_private_dir(_: &Path) {}

#[cfg(unix)]
fn set_private_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn set_private_file(_: &Path) {}
