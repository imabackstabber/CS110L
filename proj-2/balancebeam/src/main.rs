mod request;
mod response;

use clap::{Parser};
use rand::{Rng, SeedableRng};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::delay_for;
use std::collections::HashMap;

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[clap(about = "Fun with load balancing")]
struct CmdOptions {
    #[clap(
        short,
        long,
        help = "IP/port to bind to",
        default_value = "0.0.0.0:1100"
    )]
    bind: String,
    #[clap(short, long, help = "Upstream host to forward requests to")]
    upstream: Vec<String>,
    #[clap(
        long,
        help = "Perform active health checks on this interval (in seconds)",
        default_value = "10"
    )]
    active_health_check_interval: usize,
    #[clap(
    long,
    help = "Path to send request to for active health checks",
    default_value = "/"
    )]
    active_health_check_path: String,
    #[clap(
        long,
        help = "Maximum number of requests to accept per IP per minute (0 = unlimited)",
        default_value = "0"
    )]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    #[allow(dead_code)]
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks (Milestone 4)
    #[allow(dead_code)]
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    #[allow(dead_code)]
    max_requests_per_minute: usize,
    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,
    /// Tuple recording alive server number and table
    alive_addresses: Mutex<(usize, Vec<usize>)>, 
    rate_limiter: Mutex<HashMap<String,usize>>,
}

#[tokio::main] // compiler yelled:'`main` fn is not allowed to be async'
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let mut listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    // Handle incoming connections
    let upstream_len = options.upstream.len();
    let state = ProxyState {
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        alive_addresses: Mutex::new((upstream_len, vec![1;upstream_len])),
        rate_limiter: Mutex::new(HashMap::new()),
    };
    let shared_state = Arc::new(state); // we will only read it, so no Mutex is needed
    let shared_state_ref_chekcer = shared_state.clone();
    tokio::spawn(async move{
        activate_health_check(&shared_state_ref_chekcer).await;
    });
    let shared_state_ref_refresher = shared_state.clone();
    tokio::spawn(async move{
       refresh_rate_limiter(&shared_state_ref_refresher).await;
    });
    loop {
        if let Ok((stream,_)) = listener.accept().await{
            let shared_state_ref = shared_state.clone();
            tokio::spawn(async move {
                handle_connection(stream, &shared_state_ref).await;
            });
        } else{
            break;
        }
    }
}

async fn refresh_rate_limiter(state: &ProxyState){
    loop {
        delay_for(Duration::from_secs(60 as u64)).await;
        let mut rate_limiter = state.rate_limiter.lock().await;
        rate_limiter.clear();
    }
}

async fn connect_server_helper(upstream:&str) -> Result<TcpStream,()>{
    match TcpStream::connect(upstream).await{
        Ok(v) => {return Ok(v);}
        Err(_) => {
            log::error!("can't connect to server {}", upstream);
            return Err(());
        }
    }
}

async fn shakehand_with_upstream(upstream:&str, path:&str) -> Result<(),()>{
    let mut stream = connect_server_helper(upstream).await?;
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(path)
        .header("Host", upstream)
        .body(Vec::new())
        .unwrap();
    let _ = match request::write_to_stream(&request, &mut stream).await{
        Ok(v) => Ok(v),
        Err(_) => Err(())
    };
    match response::read_from_stream(&mut stream, &http::Method::GET).await{
        Ok(v) if v.status().as_u16() == 200 => {
            Ok(())
        }
        _ => {
            Err(())
        }
    }
}

async fn activate_health_check(state: &ProxyState){
    let interval = state.active_health_check_interval;
    let tot = state.upstream_addresses.len();
    loop{
        delay_for(Duration::from_secs(interval as u64)).await;
        let mut lock_ret = state.alive_addresses.lock().await;
        for i in 0..tot{
            match shakehand_with_upstream(&state.upstream_addresses[i], &state.active_health_check_path).await{
                Ok(_) => {
                    // is it marked `failed` before?
                    if lock_ret.1[i] == 0{
                        log::debug!("active check gets alive server {}:{}",i,state.upstream_addresses[i]);
                        lock_ret.1[i] = 1;
                        lock_ret.0 += 1;
                    }
                }
                Err(_) => {
                    // is it marked `available` before?
                    if lock_ret.1[i] != 0{
                        log::debug!("active check gets down server {}:{}",i,state.upstream_addresses[i]);
                        lock_ret.1[i] = 0;
                        lock_ret.0 -= 1;
                    }
                }
            }
        }
    }
}

async fn connect_to_upstream(state: &ProxyState) -> Result<TcpStream, ()> {
    loop{
        let mut rng = rand::rngs::StdRng::from_entropy();
        let upstream_idx = rng.gen_range(0, state.upstream_addresses.len());
        // get lock
        let mut lock_ret = state.alive_addresses.lock().await;
        // All server down?
        if lock_ret.0 == 0{
            log::error!("All server is down, can't connect to upstream");
            return Err(());
        }
        // test selected server is down?
        if lock_ret.1[upstream_idx] != 0{
            let upstream_ip = &state.upstream_addresses[upstream_idx];
            match TcpStream::connect(upstream_ip).await{
                Ok(v) => {return Ok(v);}
                Err(e) => {
                    log::error!("Failed to connect to upstream {}: {}", upstream_ip, e);
                    lock_ret.0 -= 1;
                    lock_ret.1[upstream_idx] = 0;
                    // will resample again
                }
            }
        }
        // else resample again
    }
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("{} <- {}", client_ip, response::format_response_line(&response));
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn check_rate_limiter(ipaddr:&str, state: &ProxyState) -> Result<(),()>{
    let mut rate_limiter = state.rate_limiter.lock().await;
    let key = String::from(ipaddr);
    let cnt = rate_limiter.entry(key).or_insert(0);
    if state.max_requests_per_minute > 0 && *cnt >= state.max_requests_per_minute{
        return Err(())
    }
    *cnt += 1;
    log::debug!("addr: {}, count: {}", ipaddr, cnt);
    Ok(())
}

async fn handle_connection(mut client_conn: TcpStream, state: &ProxyState) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = upstream_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );
        // Process rate limit
        if let Err(_e) =  check_rate_limiter(&client_ip,state).await{
            let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
            send_response(&mut client_conn, &response).await;
            continue;
        }

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!("Failed to send request to upstream {}: {}", upstream_ip, error);
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
