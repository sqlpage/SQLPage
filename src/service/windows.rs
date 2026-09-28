use super::Service;
use anyhow::Context;
use sqlpage::cli::arguments::Cli;
use std::{ffi::OsString, sync::Mutex, time::Duration};
use tokio_util::sync::CancellationToken;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

static ARGUMENTS: Mutex<Option<Cli>> = Mutex::new(None);
static RESULT: Mutex<Option<anyhow::Result<()>>> = Mutex::new(None);
static CHECKPOINT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

define_windows_service!(service_main_ffi, service_main);

pub(crate) fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let name = cli.service.clone().context("Missing service name")?;
    *ARGUMENTS.lock().unwrap() = Some(cli);
    service_dispatcher::start(&name, service_main_ffi)
        .context("Unable to connect to the Windows Service Control Manager. Start this service with Start-Service, or omit --service to run in a terminal")?;
    RESULT
        .lock()
        .unwrap()
        .take()
        .context("Windows service did not start")?
}

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service() {
        sqlpage::telemetry::windows_event_log::write(tracing::Level::ERROR, &format!("{error:#}"));
        *RESULT.lock().unwrap() = Some(Err(error));
    }
}

fn run_service() -> anyhow::Result<()> {
    let cli = ARGUMENTS
        .lock()
        .unwrap()
        .take()
        .context("Missing service arguments")?;
    let name = cli.service.as_deref().context("Missing service name")?;
    let stop = CancellationToken::new();
    let handler_stop = stop.clone();
    let status = service_control_handler::register(name, move |control| match control {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            handler_stop.cancel();
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    })?;
    let service = Service {
        stop,
        status: Some(status),
    };
    service.set_status(ServiceState::StartPending, false)?;

    let result = (|| {
        // SCM starts services in System32. Set the directory before dotenv,
        // configuration loading, or creating any runtime/worker threads.
        let root = cli
            .web_root
            .as_ref()
            .context("--service requires --web-root")?;
        anyhow::ensure!(
            root.is_absolute(),
            "--service requires an absolute --web-root"
        );
        std::env::set_current_dir(root)
            .with_context(|| format!("Unable to use service web root {}", root.display()))?;
        actix_web::rt::System::new().block_on(crate::run(cli, service.clone()))
    })();
    let failed = result.is_err();
    if let Err(error) = &result {
        sqlpage::telemetry::windows_event_log::write(tracing::Level::ERROR, &format!("{error:#}"));
    }
    // Publish the result before reporting Stopped: the dispatcher may return as
    // soon as SCM receives that status. No work may remain after this call.
    *RESULT.lock().unwrap() = Some(result);
    service.set_status(ServiceState::Stopped, failed)
}

impl Service {
    pub(super) fn set_status(&self, state: ServiceState, failed: bool) -> anyhow::Result<()> {
        if let Some(handle) = &self.status {
            let mut value = status(state, failed);
            if value.checkpoint != 0 {
                value.checkpoint = CHECKPOINT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            handle.set_service_status(value)?;
        }
        Ok(())
    }
}

fn status(state: ServiceState, failed: bool) -> ServiceStatus {
    let pending = matches!(
        state,
        ServiceState::StartPending | ServiceState::StopPending
    );
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: if state == ServiceState::Running {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else {
            ServiceControlAccept::empty()
        },
        exit_code: if failed {
            ServiceExitCode::ServiceSpecific(1)
        } else {
            ServiceExitCode::Win32(0)
        },
        checkpoint: u32::from(pending),
        wait_hint: match state {
            ServiceState::StartPending => Duration::from_secs(300),
            ServiceState::StopPending => Duration::from_secs(60),
            _ => Duration::ZERO,
        },
        process_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_status_obeys_scm_state_contract() {
        for state in [
            ServiceState::StartPending,
            ServiceState::Running,
            ServiceState::StopPending,
            ServiceState::Stopped,
        ] {
            let value = status(state, false);
            assert_eq!(
                value.controls_accepted.is_empty(),
                state != ServiceState::Running
            );
            let pending = matches!(
                state,
                ServiceState::StartPending | ServiceState::StopPending
            );
            assert_eq!(value.checkpoint > 0, pending);
            assert_eq!(!value.wait_hint.is_zero(), pending);
        }
        assert_eq!(
            status(ServiceState::Stopped, true).exit_code,
            ServiceExitCode::ServiceSpecific(1)
        );
    }
}
