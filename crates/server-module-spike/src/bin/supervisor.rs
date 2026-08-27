//! Core-side separate-process supervisor for the server-module architecture proof.

use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use omarchygs_server_module_spike::{
    CoreReceipt, HostReady, HostRequest, HostResponse, HostResult, ModuleSubject, ProofCore,
    ProofError, ProvenanceClass, fixture_request, read_bounded_artifact, read_frame, write_frame,
};
use serde::Serialize;
use uuid::Uuid;

const STARTUP_DEADLINE: Duration = Duration::from_secs(30);
const EXECUTION_DEADLINE: Duration = Duration::from_millis(500);

#[derive(Debug, Serialize)]
struct SupervisorReport {
    scenario: String,
    result: String,
    containment: String,
    startup_ms: u128,
    execution_ms: u128,
    host_rss_kib: Option<u64>,
    ready: Option<HostReady>,
    receipt: Option<CoreReceipt>,
}

enum HostMessage {
    Ready(Result<HostReady, ProofError>),
    Response(Result<HostResponse, ProofError>),
}

fn main() {
    match run() {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("supervisor report serialization failed: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("supervisor proof failed: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<SupervisorReport, ProofError> {
    let (scenario, component_path) = parse_args()?;
    let component_bytes = read_bounded_artifact(&component_path)?;
    let mut request = fixture_request(
        if scenario == "tamper" {
            b"different signed component bytes"
        } else {
            &component_bytes
        },
        ProvenanceClass::OperatorCustom {
            server_id: parse_uuid("20000000-0000-4000-8000-000000000002")?,
        },
    )?;
    if scenario == "forged-context" {
        request.event.module_id = "attacker.module".into();
        request.event.subject = ModuleSubject::Public("attacker-target".into());
    }
    let host_failure = match scenario.as_str() {
        "host-exit" => Some("exit"),
        "host-hang" => Some("hang"),
        _ => None,
    };
    let use_systemd = systemd_user_available();
    let containment = if use_systemd {
        "systemd-user-scope+bubblewrap"
    } else {
        "bubblewrap"
    };
    let started = Instant::now();
    let mut child = spawn_host(&component_path, host_failure, use_systemd)?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| ProofError::Io(std::io::Error::other("host stdin unavailable")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProofError::Io(std::io::Error::other("host stdout unavailable")))?;
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let ready = read_frame::<HostReady, _>(&mut reader);
        let ready_ok = ready.is_ok();
        if sender.send(HostMessage::Ready(ready)).is_err() || !ready_ok {
            return;
        }
        let response = read_frame::<HostResponse, _>(&mut reader);
        let _ = sender.send(HostMessage::Response(response));
    });

    let ready = match receiver.recv_timeout(STARTUP_DEADLINE) {
        Ok(HostMessage::Ready(Ok(ready))) => ready,
        Ok(HostMessage::Ready(Err(_))) | Ok(HostMessage::Response(_)) => {
            terminate(&mut child);
            return expected_startup_rejection(
                &scenario,
                containment,
                started.elapsed().as_millis(),
                None,
            );
        }
        Err(_) => {
            terminate(&mut child);
            return Err(ProofError::Execution("startup deadline exceeded".into()));
        }
    };
    validate_ready(&ready)?;
    let host_rss_kib = Some(ready.resident_kib);
    let startup_ms = started.elapsed().as_millis();
    write_frame(&mut stdin, &request)?;
    drop(stdin);
    let execution_started = Instant::now();
    let response = match receiver.recv_timeout(EXECUTION_DEADLINE) {
        Ok(HostMessage::Response(Ok(response))) => Some(response),
        Ok(HostMessage::Response(Err(_))) | Ok(HostMessage::Ready(_)) => None,
        Err(_) => {
            terminate(&mut child);
            if scenario == "host-hang" {
                return Ok(SupervisorReport {
                    scenario,
                    result: "host_timeout_contained".into(),
                    containment: containment.into(),
                    startup_ms,
                    execution_ms: execution_started.elapsed().as_millis(),
                    host_rss_kib,
                    ready: Some(ready),
                    receipt: None,
                });
            }
            return Err(ProofError::Execution("execution deadline exceeded".into()));
        }
    };
    let execution_ms = execution_started.elapsed().as_millis();
    let status = wait_for_exit(&mut child, Duration::from_millis(100))?;
    if scenario == "host-exit" {
        if status.code() == Some(70) && response.is_none() {
            return Ok(SupervisorReport {
                scenario,
                result: "host_exit_contained".into(),
                containment: containment.into(),
                startup_ms,
                execution_ms,
                host_rss_kib,
                ready: Some(ready),
                receipt: None,
            });
        }
        return Err(ProofError::Execution(
            "host exit was not contained as expected".into(),
        ));
    }
    if !status.success() {
        return Err(ProofError::Execution(
            "host exited unsuccessfully after a response".into(),
        ));
    }
    let response =
        response.ok_or_else(|| ProofError::Execution("host response was unavailable".into()))?;
    let mut core = ProofCore::at_revision(42);
    let (result, receipt) = evaluate(&scenario, &request, &response, &component_bytes, &mut core)?;
    Ok(SupervisorReport {
        scenario,
        result,
        containment: containment.into(),
        startup_ms,
        execution_ms,
        host_rss_kib,
        ready: Some(ready),
        receipt,
    })
}

fn parse_args() -> Result<(String, PathBuf), ProofError> {
    let mut args = env::args().skip(1);
    match (args.next(), args.next(), args.next(), args.next()) {
        (Some(flag), Some(scenario), Some(component), None) if flag == "--scenario" => {
            Ok((scenario, PathBuf::from(component)))
        }
        _ => Err(ProofError::Contract(
            "usage: supervisor --scenario <name> <component>".into(),
        )),
    }
}

fn spawn_host(
    component_path: &Path,
    failure: Option<&str>,
    use_systemd: bool,
) -> Result<Child, ProofError> {
    let current = env::current_exe()?;
    let host = current
        .parent()
        .ok_or_else(|| ProofError::Contract("supervisor binary has no parent".into()))?
        .join("module-host")
        .canonicalize()?;
    let component = component_path.canonicalize()?;
    let mut sandbox_args = vec![
        "--unshare-all".to_owned(),
        "--die-with-parent".to_owned(),
        "--new-session".to_owned(),
        "--clearenv".to_owned(),
        "--cap-drop".to_owned(),
        "ALL".to_owned(),
        "--ro-bind".to_owned(),
        "/usr".to_owned(),
        "/usr".to_owned(),
        "--symlink".to_owned(),
        "usr/lib".to_owned(),
        "/lib".to_owned(),
        "--symlink".to_owned(),
        "usr/lib".to_owned(),
        "/lib64".to_owned(),
        "--proc".to_owned(),
        "/proc".to_owned(),
        "--dev".to_owned(),
        "/dev".to_owned(),
        "--tmpfs".to_owned(),
        "/tmp".to_owned(),
        "--dir".to_owned(),
        "/app".to_owned(),
        "--dir".to_owned(),
        "/module".to_owned(),
        "--ro-bind".to_owned(),
        host.to_string_lossy().into_owned(),
        "/app/module-host".to_owned(),
        "--ro-bind".to_owned(),
        component.to_string_lossy().into_owned(),
        "/module/component.wat".to_owned(),
        "--setenv".to_owned(),
        "PATH".to_owned(),
        "/usr/bin".to_owned(),
        "--".to_owned(),
        "/app/module-host".to_owned(),
        "/module/component.wat".to_owned(),
    ];
    if let Some(failure) = failure {
        sandbox_args.push("--proof-failure".into());
        sandbox_args.push(failure.into());
    }
    let mut command = if use_systemd {
        let mut command = Command::new("/usr/bin/systemd-run");
        command.args([
            "--user",
            "--scope",
            "--quiet",
            "--property=MemoryMax=268435456",
            "--property=CPUQuota=50%",
            "--property=TasksMax=16",
            "/usr/bin/prlimit",
            "--nofile=64:64",
            "--",
            "/usr/bin/bwrap",
        ]);
        command
    } else {
        let mut command = Command::new("/usr/bin/prlimit");
        command.args(["--nofile=64:64", "--", "/usr/bin/bwrap"]);
        command
    };
    command
        .args(&sandbox_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if !use_systemd {
        command.env_clear();
    }
    command.spawn().map_err(ProofError::Io)
}

fn validate_ready(ready: &HostReady) -> Result<(), ProofError> {
    if ready.format != "omarchygs.server-module-host-ready/v1"
        || !ready.component_ready
        || !ready.home_absent
        || !ready.passwd_absent
        || !ready.server_environment_absent
        || !ready.loopback_only
        || ready.resident_kib == 0
        || ready.resident_kib > 256 * 1024
    {
        return Err(ProofError::Authorization(
            "sandbox readiness evidence rejected".into(),
        ));
    }
    Ok(())
}

fn expected_startup_rejection(
    scenario: &str,
    containment: &str,
    startup_ms: u128,
    host_rss_kib: Option<u64>,
) -> Result<SupervisorReport, ProofError> {
    if matches!(
        scenario,
        "forbidden-import" | "memory-hog" | "wrong-interface"
    ) {
        Ok(SupervisorReport {
            scenario: scenario.to_owned(),
            result: "startup_rejected".into(),
            containment: containment.into(),
            startup_ms,
            execution_ms: 0,
            host_rss_kib,
            ready: None,
            receipt: None,
        })
    } else {
        Err(ProofError::Execution("unexpected startup rejection".into()))
    }
}

fn evaluate(
    scenario: &str,
    request: &HostRequest,
    response: &HostResponse,
    component_bytes: &[u8],
    core: &mut ProofCore,
) -> Result<(String, Option<CoreReceipt>), ProofError> {
    match scenario {
        "tamper" | "forged-context" => match &response.outcome {
            HostResult::Rejected { code } if code == "request_rejected" => {
                Ok(("request_rejected".into(), None))
            }
            _ => Err(ProofError::Execution(
                "tampered request was not rejected".into(),
            )),
        },
        "unauthorized" => match &response.outcome {
            HostResult::Rejected { code } if code == "intent_not_granted" => {
                let receipt = core.apply(request, response, component_bytes)?;
                Ok(("unauthorized_intent_rejected".into(), Some(receipt)))
            }
            _ => Err(ProofError::Execution(
                "unauthorized intent was not rejected".into(),
            )),
        },
        "trap" | "loop" => match &response.outcome {
            HostResult::Rejected { code } if code == "module_execution_failed" => {
                let receipt = core.apply(request, response, component_bytes)?;
                Ok(("module_failure_contained".into(), Some(receipt)))
            }
            _ => Err(ProofError::Execution(
                "module failure was not contained".into(),
            )),
        },
        "noop" => {
            let receipt = core.apply(request, response, component_bytes)?;
            if receipt.committed || receipt.code != "noop" {
                return Err(ProofError::Execution("no-op outcome mismatch".into()));
            }
            Ok(("noop".into(), Some(receipt)))
        }
        "valid" => {
            let receipt = core.apply(request, response, component_bytes)?;
            if !receipt.committed || core.revision() != 43 || core.labels() != [7] {
                return Err(ProofError::Execution(
                    "allowlisted commit outcome mismatch".into(),
                ));
            }
            Ok(("core_committed_allowlisted_intent".into(), Some(receipt)))
        }
        _ => Err(ProofError::Contract("unknown supervisor scenario".into())),
    }
}

fn systemd_user_available() -> bool {
    Command::new("/usr/bin/systemctl")
        .args(["--user", "is-system-running"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn wait_for_exit(
    child: &mut Child,
    deadline: Duration,
) -> Result<std::process::ExitStatus, ProofError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started.elapsed() >= deadline {
            terminate(child);
            return Err(ProofError::Execution(
                "host did not exit after its response".into(),
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, ProofError> {
    Uuid::parse_str(value).map_err(|_| ProofError::Contract("invalid supervisor UUID".into()))
}
