//! Exercise the real executable, including OS signals and the sd-notify socket.
#![cfg(target_os = "linux")]

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::UnixDatagram,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Server {
    child: Child,
    directory: tempfile::TempDir,
    notifications: UnixDatagram,
}

impl Server {
    fn start(configuration: &serde_json::Value, sql: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("sqlpage")).unwrap();
        std::fs::write(
            directory.path().join("sqlpage/sqlpage.json"),
            configuration.to_string(),
        )
        .unwrap();
        std::fs::write(directory.path().join("index.sql"), sql).unwrap();
        let socket_path = directory.path().join("notify.sock");
        let notifications = UnixDatagram::bind(&socket_path).unwrap();
        notifications
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        let binary = std::env::var_os("SQLPAGE_BINARY")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_sqlpage").into());
        let child = Command::new(binary)
            .current_dir(directory.path())
            .env_clear()
            .env("NOTIFY_SOCKET", socket_path)
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(directory.path().join("stderr.log")).unwrap())
            .spawn()
            .unwrap();
        Self {
            child,
            directory,
            notifications,
        }
    }

    fn wait_for(&self, expected: &str) {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut buffer = [0; 2048];
        while Instant::now() < deadline {
            let size = self
                .notifications
                .recv(&mut buffer)
                .unwrap_or_else(|error| panic!("Waiting for {expected}: {error}; {}", self.logs()));
            if String::from_utf8_lossy(&buffer[..size])
                .lines()
                .any(|line| line == expected)
            {
                return;
            }
        }
        panic!("Missing {expected}: {}", self.logs());
    }

    fn signal(&self, signal: &str) {
        assert!(
            Command::new("kill")
                .args([signal, &self.child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
    }

    fn wait_for_exit(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "Service did not stop: {}",
                self.logs()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn logs(&self) -> String {
        std::fs::read_to_string(self.directory.path().join("stderr.log")).unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_address() -> std::net::SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

#[test]
fn readiness_and_signals_drain_active_requests() {
    for signal in ["-TERM", "-INT", "-QUIT"] {
        let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
        upstream.set_nonblocking(true).unwrap();
        let address = free_address();
        let mut server = Server::start(
            &serde_json::json!({"listen_on": address.to_string(), "database_url": "sqlite::memory:"}),
            &format!(
                "SELECT 'text' AS component, sqlpage.fetch('http://{}') AS contents;",
                upstream.local_addr().unwrap()
            ),
        );
        server.wait_for("READY=1");

        let mut request = TcpStream::connect(address).unwrap();
        request
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        request
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        // Wait until the real SQL request is blocked on its outbound HTTP fetch.
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut upstream_request = loop {
            match upstream.accept() {
                Ok((request, _)) => break request,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "SQL request did not start: {}",
                        server.logs()
                    );
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(error) => panic!("{error}"),
            }
        };
        upstream_request
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        let mut headers = [0; 4096];
        assert!(upstream_request.read(&mut headers).unwrap() > 0);
        server.signal(signal);
        server.wait_for("STOPPING=1");
        assert!(server.child.try_wait().unwrap().is_none());
        upstream_request.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\nrequest finished").unwrap();
        let mut response = String::new();
        request.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("request finished"), "{response}");
        assert!(server.wait_for_exit().success(), "{}", server.logs());
        assert!(server.logs().contains("Closing all database connections"));
        // A supervisor can restart immediately on the same address.
        TcpListener::bind(address).unwrap();
    }
}

#[test]
fn bind_failure_is_nonzero_and_never_reports_ready() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut server = Server::start(
        &serde_json::json!({"listen_on": listener.local_addr().unwrap().to_string(), "database_url": "sqlite::memory:"}),
        "SELECT 'text' AS component;",
    );
    assert!(!server.wait_for_exit().success());
    server.notifications.set_nonblocking(true).unwrap();
    let mut buffer = [0; 2048];
    while let Ok(size) = server.notifications.recv(&mut buffer) {
        assert!(!String::from_utf8_lossy(&buffer[..size]).contains("READY=1"));
    }
}

#[test]
fn termination_during_database_startup_exits_cleanly() {
    let database = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut server = Server::start(
        &serde_json::json!({"listen_on": "127.0.0.1:0", "database_url": format!("postgres://user:pass@{}/test", database.local_addr().unwrap())}),
        "SELECT 'text' AS component;",
    );
    server.wait_for("STATUS=Connecting to the database");
    server.signal("-TERM");
    server.wait_for("STOPPING=1");
    assert!(server.wait_for_exit().success(), "{}", server.logs());
}
