//! Small external consumer of Eylema's pinned typed local-service contract.
#![forbid(unsafe_code)]

use eylema_contracts::{
    codec::{self, Value, WireRecord},
    transport::{self, Request},
    *,
};
use std::{
    fs,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const SESSION_FILE: &str = "session.txt";
const MAX_REQUEST: usize = 64 * 1024;
const MAX_RESPONSE: usize = 4 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(5);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Eq, PartialEq)]
pub enum ClientError {
    InvalidSession,
    InvalidRequest,
    MalformedResponse,
    ServiceUnavailable,
    Remote(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSession => f.write_str("INVALID_SESSION"),
            Self::InvalidRequest => f.write_str("INVALID_REQUEST"),
            Self::MalformedResponse => f.write_str("MALFORMED_RESPONSE"),
            Self::ServiceUnavailable => f.write_str("SERVICE_UNAVAILABLE"),
            Self::Remote(code) => f.write_str(code),
        }
    }
}

impl std::error::Error for ClientError {}

struct Session {
    origin: String,
    port: u16,
    bearer: String,
    csrf: String,
}

pub struct Client {
    session: Session,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub value: u64,
    pub performed_steps: u64,
    pub charged_steps: u64,
    pub charged_events: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionSnapshot {
    pub execution_id: String,
    pub task_id: String,
    pub state: ExecutionState,
}

impl Client {
    pub fn from_environment() -> Result<Self, ClientError> {
        let directory = std::env::var_os("EYLEMA_DATA_DIR")
            .map(PathBuf::from)
            .ok_or(ClientError::InvalidSession)?;
        Self::from_data_dir(&directory)
    }

    pub fn from_data_dir(directory: &Path) -> Result<Self, ClientError> {
        let dir = fs::symlink_metadata(directory).map_err(|_| ClientError::InvalidSession)?;
        if dir.file_type().is_symlink() || !dir.is_dir() || !private_mode(dir.permissions()) {
            return Err(ClientError::InvalidSession);
        }
        let path = directory.join(SESSION_FILE);
        let metadata = fs::symlink_metadata(&path).map_err(|_| ClientError::InvalidSession)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > 2048
            || !private_mode(metadata.permissions())
        {
            return Err(ClientError::InvalidSession);
        }
        let bytes = fs::read(path).map_err(|_| ClientError::InvalidSession)?;
        if bytes.len() > 2048 {
            return Err(ClientError::InvalidSession);
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| ClientError::InvalidSession)?;
        let mut lines = text.lines();
        let origin = lines.next().ok_or(ClientError::InvalidSession)?.to_owned();
        let bearer = lines.next().ok_or(ClientError::InvalidSession)?.to_owned();
        let csrf = lines.next().ok_or(ClientError::InvalidSession)?.to_owned();
        if lines.next().is_some() || !valid_secret(&bearer) || !valid_secret(&csrf) {
            return Err(ClientError::InvalidSession);
        }
        let port = origin
            .strip_prefix("http://127.0.0.1:")
            .and_then(|port| port.parse::<u16>().ok())
            .filter(|port| *port != 0)
            .ok_or(ClientError::InvalidSession)?;
        Ok(Self {
            session: Session {
                origin,
                port,
                bearer,
                csrf,
            },
        })
    }

    pub fn request(&self, request: Request) -> Result<Value, ClientError> {
        let body = transport::encode_request(&request).map_err(|_| ClientError::InvalidRequest)?;
        if body.len() > MAX_REQUEST {
            return Err(ClientError::Remote("REQUEST_TOO_LARGE".into()));
        }
        let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.session.port);
        let mut stream = TcpStream::connect_timeout(&address.into(), IO_TIMEOUT)
            .map_err(|_| ClientError::ServiceUnavailable)?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .and_then(|_| stream.set_write_timeout(Some(IO_TIMEOUT)))
            .map_err(|_| ClientError::ServiceUnavailable)?;
        write!(
            stream,
            "POST /v1/request HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: {}\r\nAuthorization: Bearer {}\r\nX-Eylema-CSRF: {}\r\nContent-Type: application/eylema\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.session.port,
            self.session.origin,
            self.session.bearer,
            self.session.csrf,
            body.len()
        )
        .and_then(|_| stream.write_all(&body))
        .map_err(|_| ClientError::ServiceUnavailable)?;
        let mut response = Vec::new();
        stream
            .take((MAX_RESPONSE + 8192) as u64)
            .read_to_end(&mut response)
            .map_err(|_| ClientError::ServiceUnavailable)?;
        if response.len() > MAX_RESPONSE + 8192 {
            return Err(ClientError::MalformedResponse);
        }
        decode_http_response(&response)
    }

    pub fn inspect_task(&self, task_id: &TaskId) -> Result<TaskSnapshot, ClientError> {
        let value = self.request(Request::Task(task_id.clone()))?;
        let Value::List(mut fields) = value else {
            return Err(ClientError::MalformedResponse);
        };
        if fields.len() != 5 {
            return Err(ClientError::MalformedResponse);
        }
        let charged_events = number(fields.pop().ok_or(ClientError::MalformedResponse)?)?;
        let charged_steps = number(fields.pop().ok_or(ClientError::MalformedResponse)?)?;
        let performed_steps = number(fields.pop().ok_or(ClientError::MalformedResponse)?)?;
        let value = number(fields.pop().ok_or(ClientError::MalformedResponse)?)?;
        let Value::Bytes(record) = fields.pop().ok_or(ClientError::MalformedResponse)? else {
            return Err(ClientError::MalformedResponse);
        };
        let WireRecord::Task(task) =
            codec::decode_record(&record).map_err(|_| ClientError::MalformedResponse)?
        else {
            return Err(ClientError::MalformedResponse);
        };
        Ok(TaskSnapshot {
            task_id: task.fields().task_id.as_str().to_owned(),
            value,
            performed_steps,
            charged_steps,
            charged_events,
        })
    }

    pub fn inspect_execution(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<ExecutionSnapshot, ClientError> {
        let value = self.request(Request::Execution(execution_id.clone()))?;
        let Value::Bytes(record) = value else {
            return Err(ClientError::MalformedResponse);
        };
        let WireRecord::Execution { execution, .. } =
            codec::decode_record(&record).map_err(|_| ClientError::MalformedResponse)?
        else {
            return Err(ClientError::MalformedResponse);
        };
        let fields = execution.fields();
        Ok(ExecutionSnapshot {
            execution_id: fields.execution_id.as_str().to_owned(),
            task_id: fields.task_id.as_str().to_owned(),
            state: fields.state,
        })
    }

    pub fn cancel(&self, execution_id: ExecutionId) -> Result<Value, ClientError> {
        self.request(Request::Cancel(execution_id))
    }
}

pub fn create_and_run(delta: u64) -> Result<(TaskSnapshot, ExecutionSnapshot), ClientError> {
    let client = Client::from_environment()?;
    create_and_run_with_client(&client, delta)
}

pub fn create_and_run_with_client(
    client: &Client,
    delta: u64,
) -> Result<(TaskSnapshot, ExecutionSnapshot), ClientError> {
    let (task, request) = prepare_demo(delta, "task:state")?;
    let task_id = task.fields().task_id.clone();
    client.request(Request::Create(task))?;
    let response = client.request(request)?;
    let Value::Text(execution) = response else {
        return Err(ClientError::MalformedResponse);
    };
    let execution_id = ExecutionId::new(execution).map_err(|_| ClientError::MalformedResponse)?;
    let deadline = SystemTime::now() + IO_TIMEOUT;
    loop {
        let snapshot = client.inspect_task(&task_id)?;
        let execution_snapshot = client.inspect_execution(&execution_id)?;
        if execution_snapshot.state.is_terminal() {
            return Ok((snapshot, execution_snapshot));
        }
        if SystemTime::now() >= deadline {
            return Err(ClientError::Remote("OUTCOME_UNKNOWN".into()));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn prepare_demo(delta: u64, capability: &str) -> Result<(TaskSpec, Request), ClientError> {
    let payload =
        codec::encode(&Value::Unsigned(delta)).map_err(|_| ClientError::InvalidRequest)?;
    prepare_request(capability, payload)
}

pub fn prepare_request_payload(
    capability: &str,
    payload: Vec<u8>,
) -> Result<(TaskSpec, Request), ClientError> {
    prepare_request(capability, payload)
}

fn prepare_request(capability: &str, payload: Vec<u8>) -> Result<(TaskSpec, Request), ClientError> {
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(ClientError::InvalidRequest);
    }
    let delta = match codec::decode(&payload) {
        Ok(Value::Unsigned(value)) => value,
        _ => 1,
    };
    if !(1..=1_000_000).contains(&delta) {
        return Err(ClientError::InvalidRequest);
    }
    let serial = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let stem = format!("g17-{}-{serial}", std::process::id());
    let task_id = TaskId::new(format!("task:{stem}")).map_err(|_| ClientError::InvalidRequest)?;
    let revision = Revision::new(1).map_err(|_| ClientError::InvalidRequest)?;
    let request_context = context(&stem)?;
    let scope = ResourceScope::new(vec![
        ResourceId::new("state:cli").map_err(|_| ClientError::InvalidRequest)?,
    ])
    .map_err(|_| ClientError::InvalidRequest)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ClientError::InvalidRequest)?
        .as_millis() as u64;
    let budget = Budget::new(
        1,
        12,
        payload.len().max(1024) as u32,
        1,
        Timestamp::new(now.saturating_add(60_000)).map_err(|_| ClientError::InvalidRequest)?,
    )
    .map_err(|_| ClientError::InvalidRequest)?;
    let target = ExecutionTarget::new(
        ExecutionContext::NativeMacos,
        Backend::NativeCpu,
        GpuApi::None,
    )
    .map_err(|_| ClientError::InvalidRequest)?;
    let task = TaskSpec::new(TaskDraft {
        context: request_context.clone(),
        task_id: task_id.clone(),
        revision,
        objective: "Advance a bounded local task value".into(),
        resource_scope: scope.clone(),
        budget,
        target,
    })
    .map_err(|_| ClientError::InvalidRequest)?;
    let action = ActionSpec::new(ActionDraft {
        context: request_context.clone(),
        action_id: ActionId::new(format!("action:{stem}"))
            .map_err(|_| ClientError::InvalidRequest)?,
        task_id: task_id.clone(),
        operation_id: OperationId::new("addTaskValue").map_err(|_| ClientError::InvalidRequest)?,
        operation_version: ContractVersion::CURRENT,
        side_effect_class: SideEffectClass::RecordMutation,
        resource_scope: scope.clone(),
        expected_revision: Some(revision),
        authority: AuthorityRequirement::new(
            PrincipalRef::new("principal:local-cli").map_err(|_| ClientError::InvalidRequest)?,
            revision,
            scope,
            vec![Permission::ExecuteAction],
            OversightRequirements::new(vec![], true, false, None)
                .map_err(|_| ClientError::InvalidRequest)?,
        )
        .map_err(|_| ClientError::InvalidRequest)?,
        required_capabilities: vec![
            CapabilityId::new(capability).map_err(|_| ClientError::InvalidRequest)?,
        ],
        budget,
        target,
        idempotency_key: IdempotencyKey::new(format!("{stem}:step:1"))
            .map_err(|_| ClientError::InvalidRequest)?,
        payload,
    })
    .map_err(|_| ClientError::InvalidRequest)?;
    let proposal_id =
        ProposalId::new(format!("proposal:{stem}")).map_err(|_| ClientError::InvalidRequest)?;
    let proposal = ActionProposal::new(
        ProposalDraft {
            context: request_context,
            proposal_id,
            task_id: task_id.clone(),
            plan_revision: revision,
            state_revision: revision,
            policy_revision: revision,
            actions: vec![action],
        },
        &task,
    )
    .map_err(|_| ClientError::InvalidRequest)?;
    Ok((
        task.clone(),
        Request::Admit {
            task,
            proposal,
            approvals: vec![],
        },
    ))
}

fn context(stem: &str) -> Result<RequestContext, ClientError> {
    Ok(RequestContext {
        request_id: RequestId::new(format!("request:{stem}"))
            .map_err(|_| ClientError::InvalidRequest)?,
        correlation_id: CorrelationId::new(format!("correlation:{stem}"))
            .map_err(|_| ClientError::InvalidRequest)?,
        contract_version: ContractVersion::CURRENT,
    })
}

fn number(value: Value) -> Result<u64, ClientError> {
    if let Value::Unsigned(value) = value {
        Ok(value)
    } else {
        Err(ClientError::MalformedResponse)
    }
}

fn valid_secret(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(unix)]
fn private_mode(permissions: std::fs::Permissions) -> bool {
    use std::os::unix::fs::PermissionsExt;
    permissions.mode() & 0o077 == 0
}

#[cfg(not(unix))]
fn private_mode(_: std::fs::Permissions) -> bool {
    false
}

fn decode_http_response(response: &[u8]) -> Result<Value, ClientError> {
    let split = response
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .ok_or(ClientError::MalformedResponse)?;
    let headers =
        std::str::from_utf8(&response[..split]).map_err(|_| ClientError::MalformedResponse)?;
    let mut lines = headers.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or(ClientError::MalformedResponse)?;
    let length = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name.eq_ignore_ascii_case("content-length"))
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .ok_or(ClientError::MalformedResponse)?;
    if length > MAX_RESPONSE || response.len() != split + 4 + length {
        return Err(ClientError::MalformedResponse);
    }
    let decoded = transport::decode_response(&response[split + 4..]).map_err(|error| match error {
        transport::TransportError::Malformed => ClientError::MalformedResponse,
        transport::TransportError::Remote(code) => ClientError::Remote(code),
    });
    if !(200..300).contains(&status) {
        return match decoded {
            Err(error @ ClientError::Remote(_)) => Err(error),
            _ => Err(ClientError::MalformedResponse),
        };
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    fn private_test_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eylema-examples-test-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create private test directory");
        set_mode(&path, 0o700);
        path
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set private mode");
    }

    #[cfg(not(unix))]
    fn set_mode(_: &Path, _: u32) {}

    fn write_session(directory: &Path, origin: &str) {
        let path = directory.join(SESSION_FILE);
        fs::write(
            path,
            format!("{origin}\n{}\n{}\n", "a".repeat(64), "b".repeat(64)),
        )
        .expect("write session fixture");
        set_mode(&directory.join(SESSION_FILE), 0o600);
    }

    fn http_response(status: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);
        response
    }

    #[test]
    fn private_loopback_session_is_required_and_credentials_are_not_debuggable() {
        let directory = private_test_dir();
        write_session(&directory, "http://127.0.0.1:51123");
        assert!(Client::from_data_dir(&directory).is_ok());
        let _ = fs::remove_dir_all(&directory);

        let directory = private_test_dir();
        write_session(&directory, "http://192.0.2.1:51123");
        assert_eq!(
            Client::from_data_dir(&directory).err(),
            Some(ClientError::InvalidSession)
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn malformed_and_non_success_http_envelopes_fail_closed() {
        let body = transport::encode_response(Value::Unsigned(7)).expect("encode response fixture");
        assert_eq!(
            decode_http_response(&http_response("200 OK", &body)),
            Ok(Value::Unsigned(7))
        );
        let denied = transport::encode_error("FORBIDDEN").expect("encode error fixture");
        assert_eq!(
            decode_http_response(&http_response("200 OK", &denied)),
            Err(ClientError::Remote("FORBIDDEN".into()))
        );
        assert_eq!(
            decode_http_response(&http_response("403 Forbidden", &body)),
            Err(ClientError::MalformedResponse)
        );
        assert_eq!(
            decode_http_response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n0"),
            Err(ClientError::MalformedResponse)
        );
    }

    #[test]
    fn request_size_boundary_is_enforced_before_socket_use() {
        let directory = private_test_dir();
        write_session(&directory, "http://127.0.0.1:51123");
        let client = Client::from_data_dir(&directory).expect("private loopback session");
        let (_, request) = prepare_request_payload("task:state", vec![0; MAX_REQUEST + 1024])
            .expect("bounded typed proposal fixture");
        assert_eq!(
            client.request(request),
            Err(ClientError::Remote("REQUEST_TOO_LARGE".into()))
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn demo_rejects_out_of_range_input_before_creating_authority_or_state() {
        assert_eq!(
            prepare_demo(0, "task:state").err(),
            Some(ClientError::InvalidRequest)
        );
        assert_eq!(
            prepare_demo(1_000_001, "task:state").err(),
            Some(ClientError::InvalidRequest)
        );
    }
}
