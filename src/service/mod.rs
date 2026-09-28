//! Process lifecycle integration. Kept in the executable so embedding `SQLPage` does
//! not install process-wide signal handlers or contact a service manager.

use tokio_util::sync::CancellationToken;

#[cfg(windows)]
pub(super) mod windows;

#[derive(Clone)]
pub(super) struct Service {
    pub(super) stop: CancellationToken,
    #[cfg(windows)]
    pub(super) status: Option<windows_service::service_control_handler::ServiceStatusHandle>,
}

impl Service {
    pub(super) fn console() -> anyhow::Result<Self> {
        let service = Self {
            stop: CancellationToken::new(),
            #[cfg(windows)]
            status: None,
        };
        let stop = service.stop.clone();
        // Register before database connections or migrations can delay startup.
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut terminate = signal(SignalKind::terminate())?;
            let mut interrupt = signal(SignalKind::interrupt())?;
            let mut quit = signal(SignalKind::quit())?;
            tokio::spawn(async move {
                tokio::select! {
                    _ = terminate.recv() => {},
                    _ = interrupt.recv() => {},
                    _ = quit.recv() => {},
                }
                stop.cancel();
            });
        }
        #[cfg(windows)]
        {
            let mut interrupt = tokio::signal::windows::ctrl_c()?;
            let mut terminate = tokio::signal::windows::ctrl_break()?;
            tokio::spawn(async move {
                tokio::select! {
                    _ = interrupt.recv() => {},
                    _ = terminate.recv() => {},
                }
                stop.cancel();
            });
        }
        Ok(service)
    }

    #[cfg_attr(
        not(windows),
        expect(
            clippy::unused_self,
            reason = "Windows service state selects the log destination"
        )
    )]
    pub(super) fn init_telemetry(&self) -> anyhow::Result<bool> {
        #[cfg(windows)]
        if self.status.is_some() {
            return sqlpage::telemetry::init_windows_service_telemetry();
        }
        sqlpage::telemetry::init_telemetry()
    }

    #[cfg_attr(
        not(windows),
        expect(
            clippy::unused_self,
            reason = "Windows services report startup progress"
        )
    )]
    pub(super) fn starting(&self, message: &str) -> anyhow::Result<()> {
        log::info!("{message}");
        #[cfg(target_os = "linux")]
        sd_notify::notify(false, &[sd_notify::NotifyState::Status(message)])?;
        #[cfg(windows)]
        self.set_status(windows_service::service::ServiceState::StartPending, false)?;
        Ok(())
    }

    pub(super) fn ready(&self) -> anyhow::Result<()> {
        if self.stop.is_cancelled() {
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        sd_notify::notify(
            false,
            &[
                sd_notify::NotifyState::Ready,
                sd_notify::NotifyState::Status("Serving requests"),
            ],
        )?;
        #[cfg(windows)]
        self.set_status(windows_service::service::ServiceState::Running, false)?;
        Ok(())
    }

    #[cfg_attr(
        not(windows),
        expect(
            clippy::unused_self,
            reason = "Windows service state holds the status handle"
        )
    )]
    pub(super) fn stopping(&self) -> anyhow::Result<()> {
        log::info!("Stopping SQLPage; waiting for active requests to finish");
        #[cfg(target_os = "linux")]
        sd_notify::notify(
            false,
            &[
                sd_notify::NotifyState::Stopping,
                sd_notify::NotifyState::Status("Draining requests and closing the database"),
            ],
        )?;
        #[cfg(windows)]
        self.set_status(windows_service::service::ServiceState::StopPending, false)?;
        Ok(())
    }
}
