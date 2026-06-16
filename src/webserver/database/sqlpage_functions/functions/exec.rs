use std::borrow::Cow;

use anyhow::Context;
use tracing::Instrument;

use crate::webserver::http_request_info::RequestInfo;

/// Executes an external command and returns its output.
pub(super) async fn exec<'a>(
    request: &'a RequestInfo,
    program_name: Cow<'a, str>,
    args: Vec<Cow<'a, str>>,
) -> anyhow::Result<String> {
    if !request.app_state.config.allow_exec {
        anyhow::bail!("The sqlpage.exec() function is disabled in the configuration, for security reasons.
        Make sure you understand the security implications before enabling it, and never allow user input to be passed as the first argument to this function.
        You can enable it by setting the allow_exec option to true in the sqlpage.json configuration file.")
    }
    let exec_span = tracing::info_span!(
        "subprocess",
        otel.name = format!("EXEC {program_name}"),
        process.command = %program_name,
        process.args_count = args.len(),
    );
    let res = tokio::process::Command::new(&*program_name)
        .args(args.iter().map(|x| &**x))
        .output()
        .instrument(exec_span)
        .await
        .with_context(|| {
            let mut s = format!("Unable to execute command: {program_name}");
            for arg in args {
                s.push(' ');
                s.push_str(&arg);
            }
            s
        })?;
    if !res.status.success() {
        anyhow::bail!(
            "Command '{program_name}' failed with exit code {}: {}",
            res.status,
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&res.stdout).into_owned())
}
