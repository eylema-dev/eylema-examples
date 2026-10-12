use eylema_contracts::{ExecutionId, TaskId};
use eylema_task_console_example::{Client, create_and_run};

fn main() {
    if let Err(error) = run() {
        eprintln!("{{\"status\":\"error\",\"code\":\"{error}\"}}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("demo") => {
            let delta = args
                .next()
                .unwrap_or_else(|| "1".into())
                .parse::<u64>()
                .map_err(|_| "INVALID_ARGUMENT".to_string())?;
            if args.next().is_some() {
                return Err("INVALID_ARGUMENT".into());
            }
            let (task, execution) = create_and_run(delta).map_err(|error| error.to_string())?;
            println!(
                "{{\"status\":\"ok\",\"task_id\":\"{}\",\"execution_id\":\"{}\",\"state\":\"{:?}\",\"value\":{},\"steps\":{}}}",
                task.task_id,
                execution.execution_id,
                execution.state,
                task.value,
                task.performed_steps
            );
        }
        Some("inspect") => {
            let kind = args.next().ok_or("INVALID_ARGUMENT")?;
            let id = args.next().ok_or("INVALID_ARGUMENT")?;
            if args.next().is_some() {
                return Err("INVALID_ARGUMENT".into());
            }
            let client = Client::from_environment().map_err(|error| error.to_string())?;
            match kind.as_str() {
                "task" => {
                    let task_id = TaskId::new(id).map_err(|_| "INVALID_REQUEST".to_string())?;
                    let snapshot = client
                        .inspect_task(&task_id)
                        .map_err(|error| error.to_string())?;
                    println!(
                        "{{\"status\":\"ok\",\"kind\":\"task\",\"task_id\":\"{}\",\"value\":{},\"steps\":{}}}",
                        snapshot.task_id, snapshot.value, snapshot.performed_steps
                    );
                }
                "execution" => {
                    let execution_id =
                        ExecutionId::new(id).map_err(|_| "INVALID_REQUEST".to_string())?;
                    let snapshot = client
                        .inspect_execution(&execution_id)
                        .map_err(|error| error.to_string())?;
                    println!(
                        "{{\"status\":\"ok\",\"kind\":\"execution\",\"execution_id\":\"{}\",\"task_id\":\"{}\",\"state\":\"{:?}\"}}",
                        snapshot.execution_id, snapshot.task_id, snapshot.state
                    );
                }
                _ => return Err("INVALID_ARGUMENT".into()),
            }
        }
        _ => return Err("usage: task-console <demo [delta]|inspect <task|execution> <id>>".into()),
    }
    Ok(())
}
