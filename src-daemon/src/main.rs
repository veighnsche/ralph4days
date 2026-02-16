use core_errors::{codes, err_string, RalphResult};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod commands;
mod event_sink;
mod rpc_codec;
mod state;
mod ws_server;

fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("ralphd=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}

fn parse_bind_addr() -> RalphResult<std::net::SocketAddr> {
    let mut args = std::env::args().skip(1);
    let mut bind = "127.0.0.1:9944".to_owned();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                bind = args.next().ok_or_else(|| {
                    err_string(
                        codes::INTERNAL,
                        "Missing value for --bind (expected HOST:PORT)",
                    )
                })?;
            }
            "--help" | "-h" => {
                eprintln!("Usage: ralphd [--bind HOST:PORT]");
                std::process::exit(0);
            }
            other => {
                return Err(err_string(
                    codes::INTERNAL,
                    format!("Unknown CLI argument: {other}"),
                ));
            }
        }
    }

    parse_bind_value(&bind)
}

fn parse_bind_value(bind: &str) -> RalphResult<SocketAddr> {
    let parsed = bind.parse().map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!("Invalid --bind value '{bind}': {error}"),
        )
    })?;

    ensure_loopback_bind(parsed)
}

fn ensure_loopback_bind(bind: SocketAddr) -> RalphResult<SocketAddr> {
    if bind.ip().is_loopback() {
        return Ok(bind);
    }

    Err(err_string(
        codes::INTERNAL,
        format!(
            "Refusing non-loopback bind '{bind}'. Ralphd remote access is SSH-tunnel-only; bind to 127.0.0.1/::1 and forward via SSH."
        ),
    ))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_tracing();

    let bind = parse_bind_addr().map_err(|e| {
        eprintln!("{e}");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
    })?;

    let listener = TcpListener::bind(bind).await?;
    let local = listener.local_addr()?;
    tracing::info!(%local, "ralphd listening");
    if std::env::var_os("RALPHD_PRINT_LISTEN_ADDR").is_some() {
        println!("RALPHD_LISTEN_ADDR={local}");
    }

    let state = Arc::new(state::AppState::default());
    service_runtime::diagnostics::register_sink(Arc::clone(&state.event_sink));

    // Start API server for MCP signal communication.
    let port = service_tasks::api_server::start_api_server(Arc::clone(&state.event_sink))
        .await
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))?;
    *state.api_server_port.lock().unwrap() = Some(port);
    tracing::info!(port, "api-server started");

    loop {
        let (stream, peer) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(error) = ws_server::handle_connection(stream, state).await {
                tracing::warn!(%peer, error = %error, "ralphd connection closed with error");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{ensure_loopback_bind, parse_bind_value};

    #[test]
    fn parse_bind_value_accepts_ipv4_loopback() {
        let bind =
            parse_bind_value("127.0.0.1:9944").expect("expected ipv4 loopback bind to be valid");
        assert_eq!(bind.to_string(), "127.0.0.1:9944");
    }

    #[test]
    fn parse_bind_value_accepts_ipv6_loopback() {
        let bind = parse_bind_value("[::1]:9944").expect("expected ipv6 loopback bind to be valid");
        assert_eq!(bind.to_string(), "[::1]:9944");
    }

    #[test]
    fn parse_bind_value_rejects_non_loopback_ipv4() {
        let error = parse_bind_value("0.0.0.0:9944")
            .expect_err("expected non-loopback bind to be rejected");
        assert!(
            error.to_string().contains("SSH-tunnel-only"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn ensure_loopback_bind_rejects_non_loopback_ipv6() {
        let bind = "[2001:db8::1]:9944".parse().expect("valid socket addr");
        let error =
            ensure_loopback_bind(bind).expect_err("expected non-loopback bind to be rejected");
        assert!(
            error.to_string().contains("bind to 127.0.0.1/::1"),
            "unexpected error: {error}"
        );
    }
}
