//! PPC (Programmable Processing Core) Linker
//!
//! Connects the For_Your_File application to the PPC Central Processing System
//! running on 127.0.0.1:9527. Handles:
//! - TCP connection management
//! - App registration (REGISTER_APP) and authentication (AUTH)
//! - PPC version verification (must be exactly 0.0.7)
//! - PPC error code mapping to human-readable messages
//! - Command sending and response parsing
//!
//! This module does NOT modify any PPC source files. It acts purely as a
//! client that communicates with PPC over TCP.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// PPC server address (localhost only, as PPC binds to 127.0.0.1).
const PPC_HOST: &str = "127.0.0.1";
/// PPC server port.
const PPC_PORT: u16 = 9527;

/// The app ID under which For_Your_File registers with PPC.
const APP_ID: &str = "fyf";

/// For_Your_File's own version, sent during registration.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported PPC version.
const PPC_MIN_VERSION: &str = "0.0.7";
/// Maximum supported PPC version.
const PPC_MAX_VERSION: &str = "0.0.8";

/// TCP read/write timeout in seconds.
const TIMEOUT_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// Connection state (held in a Tauri State)
// ---------------------------------------------------------------------------

/// Holds the authenticated session state after a successful registration +
/// AUTH cycle. The hash is needed to re-authenticate on new TCP connections
/// (PPC requires AUTH on every new TCP connection).
#[derive(Clone, serde::Serialize)]
pub struct PpcSession {
    /// Whether PPC is currently connected and authenticated.
    pub connected: bool,
    /// The app hash returned by PPC during registration (used for AUTH).
    pub app_hash: Option<String>,
    /// The PPC version reported by the server.
    pub ppc_version: Option<String>,
    /// Human-readable status message.
    pub status_message: String,
    /// Last error code from PPC (hex string like "0x10000").
    pub last_error_code: Option<String>,
}

impl Default for PpcSession {
    fn default() -> Self {
        Self {
            connected: false,
            app_hash: None,
            ppc_version: None,
            status_message: "Not connected".to_string(),
            last_error_code: None,
        }
    }
}

/// Tauri-managed state wrapping the session.
pub struct PpcState {
    pub session: Mutex<PpcSession>,
}

impl PpcState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(PpcSession::default()),
        }
    }
}

// ---------------------------------------------------------------------------
// Error code mapping
// ---------------------------------------------------------------------------

/// Map a PPC hex status code to a human-readable message in English.
///
/// The mapping follows the `app_code.json` definitions in the PPC project.
/// Unknown codes return a generic message.
pub fn map_ppc_code(code: &str) -> String {
    match code {
        // Success codes (0x0xxxx)
        "0x00000" => "Success".to_string(),
        "0x00001" => "App registered successfully".to_string(),
        "0x00002" => "App updated successfully".to_string(),
        "0x00003" => "DLL loaded successfully".to_string(),
        "0x00004" => "Command executed successfully".to_string(),

        // Error codes (0x1xxxx)
        "0x10000" => "Unknown error".to_string(),
        "0x10001" => "Unsupported command".to_string(),
        "0x10002" => "Resource not found".to_string(),
        "0x10003" => "Signature verification failed".to_string(),
        "0x10004" => "Hash mismatch".to_string(),
        "0x10005" => "Registration failed".to_string(),
        "0x10006" => "Update failed".to_string(),
        "0x10007" => "App not registered or not authenticated".to_string(),
        "0x10008" => "Invalid parameters".to_string(),
        "0x10009" => "File not found".to_string(),
        "0x10010" => "Permission denied".to_string(),
        "0x10011" => "DLL not found".to_string(),
        "0x10012" => "DLL load failed".to_string(),
        "0x10013" => "Config parse failed".to_string(),
        "0x10014" => "Database error".to_string(),
        "0x10015" => "Connection failed".to_string(),
        "0x10016" => "Timeout".to_string(),
        "0x10017" => "PPC server not running".to_string(),
        "0x10018" => "PPC authentication failed".to_string(),
        "0x10019" => "PPC version not supported".to_string(),
        "0x10020" => "PPC response invalid".to_string(),
        "0x10021" => "Window pin failed".to_string(),
        "0x10022" => "Window not found".to_string(),
        "0x10023" => "Hotkey registration failed".to_string(),
        "0x10024" => "Config save failed".to_string(),
        "0x10025" => "Config load failed".to_string(),
        "0x10026" => "State persistence failed".to_string(),
        "0x10027" => "Administrator privileges required".to_string(),
        "0x10028" => "Localization load failed".to_string(),
        "0x10029" => "Single instance violation".to_string(),
        "0x10030" => "Hook installation failed".to_string(),
        "0x10031" => "Log write failed".to_string(),
        "0x10032" => "DLL call failed".to_string(),

        // Status codes (0x2xxxx)
        "0x20000" => "Waiting for signature".to_string(),
        "0x20001" => "Waiting for app info".to_string(),
        "0x20002" => "Processing".to_string(),
        "0x20003" => "Initializing".to_string(),
        "0x20004" => "Loading config".to_string(),
        "0x20005" => "Executing command".to_string(),
        "0x20006" => "Connecting".to_string(),
        "0x20007" => "Registering".to_string(),
        "0x20008" => "Authenticating".to_string(),
        "0x20009" => "Reconnecting".to_string(),

        // Warning codes (0x3xxxx)
        "0x30000" => "Deprecated command warning".to_string(),
        "0x30001" => "Old version warning".to_string(),
        "0x30002" => "High memory usage warning".to_string(),
        "0x30003" => "PPC reconnect warning".to_string(),
        "0x30004" => "Fallback behavior warning".to_string(),
        "0x30005" => "Language fallback warning".to_string(),

        _ => format!("Unknown status code: {}", code),
    }
}

/// Check if a PPC code represents success (starts with "0x0").
pub fn is_success_code(code: &str) -> bool {
    code.to_lowercase().starts_with("0x0")
}

/// Check if a PPC code represents an error (starts with "0x1").
#[allow(dead_code)]
pub fn is_error_code(code: &str) -> bool {
    code.to_lowercase().starts_with("0x1")
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

/// Parse a version string "x.y.z" into a Vec<u32>.
fn parse_version(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
}

/// Check if a PPC version is within the supported range.
/// Min = 0.0.7, Max = 0.0.7 (only 0.0.7 is supported).
pub fn is_version_supported(version: &str) -> bool {
    let v = parse_version(version);
    let min = parse_version(PPC_MIN_VERSION);
    let max = parse_version(PPC_MAX_VERSION);

    if v.len() != 3 || min.len() != 3 || max.len() != 3 {
        // Fallback to string comparison if parsing fails
        return version == PPC_MIN_VERSION;
    }

    // Compare: min <= v <= max
    let v_parts = (v[0], v[1], v[2]);
    let min_parts = (min[0], min[1], min[2]);
    let max_parts = (max[0], max[1], max[2]);

    v_parts >= min_parts && v_parts <= max_parts
}

// ---------------------------------------------------------------------------
// TCP communication
// ---------------------------------------------------------------------------

/// Send a command to PPC and return the raw response string.
fn send_command(command: &str) -> Result<String, String> {
    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(TIMEOUT_SECS),
    )
    .map_err(|e| format!("Failed to connect to PPC ({}): {}", addr, e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    // Send command (PPC reads raw bytes, no line-ending required but \n is safe)
    let cmd_bytes = format!("{}\n", command);
    stream
        .write_all(cmd_bytes.as_bytes())
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut buf = [0u8; 4096];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    String::from_utf8(buf[..n].to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
}

/// Parse the first line of a PPC response as the status code.
fn parse_status_code(response: &str) -> Option<String> {
    response.lines().next().map(|line| line.trim().to_string())
}

/// Parse the second line of a PPC response (often the payload).
fn parse_payload(response: &str) -> Option<String> {
    response.lines().nth(1).map(|line| line.trim().to_string())
}

// ---------------------------------------------------------------------------
// Registration & Authentication
// ---------------------------------------------------------------------------

/// Get the current executable path for registration.
fn get_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

/// Register the For_Your_File app with PPC.
/// Returns the app hash on success.
pub fn register_with_ppc() -> Result<String, String> {
    let exe_path = get_exe_path();
    let command = format!("REGISTER_APP {}|{}|{}", APP_ID, APP_VERSION, exe_path);
    log::info!("PPC register: {}", command);

    let response = send_command(&command)?;
    let code = parse_status_code(&response).unwrap_or_default();

    if is_success_code(&code) {
        // Payload is the hash on the second line
        let hash = parse_payload(&response)
            .ok_or_else(|| format!("Registration succeeded but no hash returned: {}", response))?;
        log::info!("PPC registration succeeded, hash={}", hash);
        Ok(hash)
    } else {
        let msg = map_ppc_code(&code);
        Err(format!("PPC registration failed: {} ({})", msg, code))
    }
}

/// Authenticate with PPC using the stored hash.
/// Returns Ok(()) on success.
pub fn authenticate_with_ppc(hash: &str) -> Result<(), String> {
    let command = format!("AUTH {}", hash);
    let response = send_command(&command)?;
    let code = parse_status_code(&response).unwrap_or_default();

    if is_success_code(&code) {
        log::info!("PPC authentication succeeded");
        Ok(())
    } else {
        let msg = map_ppc_code(&code);
        Err(format!("PPC authentication failed: {} ({})", msg, code))
    }
}

/// Query the PPC version and verify it's within the supported range.
pub fn check_ppc_version() -> Result<String, String> {
    let response = send_command("PPCVERSION")?;
    let code = parse_status_code(&response).unwrap_or_default();

    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("Failed to query PPC version: {} ({})", msg, code));
    }

    // Response format: "0x00000\nPPC Version: 0.0.7\n"
    // Extract version from the payload line
    let payload = parse_payload(&response).unwrap_or_default();
    let version = payload
        .replace("PPC Version:", "")
        .replace("PPC version:", "")
        .trim()
        .to_string();

    if version.is_empty() {
        return Err(format!("Cannot parse PPC version: {}", response));
    }

    if !is_version_supported(&version) {
        return Err(format!(
            "PPC version {} not in supported range (only {}~{})",
            version, PPC_MIN_VERSION, PPC_MAX_VERSION
        ));
    }

    log::info!("PPC version check passed: {}", version);
    Ok(version)
}

/// Ping PPC to check if it's running.
pub fn ping_ppc() -> Result<bool, String> {
    match send_command("PING") {
        Ok(response) => {
            let code = parse_status_code(&response).unwrap_or_default();
            Ok(is_success_code(&code))
        }
        Err(_) => Ok(false),
    }
}

// ---------------------------------------------------------------------------
// Data directory resolution (PPC-managed config path)
// ---------------------------------------------------------------------------

/// Cached PPC base path once successfully resolved, so we don't repeat the
/// register→auth→PPCPATH round-trip on every data-dir lookup.
static PPC_BASE_PATH_CACHE: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

/// Cooldown deadline after a failed PPC connection attempt.
/// When the current time is before this instant, skip the connection attempt
/// to avoid repeatedly blocking on an unavailable PPC server during startup.
static PPC_FAIL_COOLDOWN: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Duration to wait before retrying a failed PPC connection (30 seconds).
const PPC_FAIL_COOLDOWN_SECS: u64 = 30;

/// Query the PPC installation directory via the PPCPATH command.
///
/// PPCPATH is not a public command, so this performs REGISTER_APP → AUTH →
/// PPCPATH on a single TCP connection. Returns the PPC base directory
/// (the directory containing ppc.exe).
#[allow(dead_code)]
fn query_ppc_base_path() -> Result<std::path::PathBuf, String> {
    // Fast path: cached value from a previous successful resolution
    if let Some(cached) = PPC_BASE_PATH_CACHE.lock().unwrap().as_ref() {
        return Ok(cached.clone());
    }

    // Check cooldown: skip connection attempt if we recently failed
    {
        let cooldown = PPC_FAIL_COOLDOWN.lock().unwrap();
        if let Some(deadline) = cooldown.as_ref() {
            if std::time::Instant::now() < *deadline {
                log::debug!("PPC connection in cooldown, skipping query");
                return Err("PPC not reachable (cooldown active)".to_string());
            }
        }
    }

    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    // Use a short connection timeout (500ms) to avoid blocking app startup
    // when PPC is not yet running. Once connected, read/write timeouts
    // remain at TIMEOUT_SECS for reliable command exchange.
    let connect_result = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_millis(500),
    );

    let mut stream = match connect_result {
        Ok(stream) => stream,
        Err(e) => {
            // Record cooldown on failure so subsequent calls don't block
            *PPC_FAIL_COOLDOWN.lock().unwrap() =
                Some(std::time::Instant::now() + Duration::from_secs(PPC_FAIL_COOLDOWN_SECS));
            return Err(format!("PPC not reachable: {}", e));
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    let mut buf = [0u8; 4096];

    // Step 1: REGISTER_APP (public command, no auth required)
    let exe_path = get_exe_path();
    let register_cmd = format!("REGISTER_APP {}|{}|{}\n", APP_ID, APP_VERSION, exe_path);
    stream
        .write_all(register_cmd.as_bytes())
        .map_err(|e| format!("Failed to send REGISTER_APP: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read REGISTER_APP response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("PPC register failed: {} ({})", msg, code));
    }
    let hash = parse_payload(&response).unwrap_or_default();
    if hash.is_empty() {
        return Err("PPC register returned empty hash".to_string());
    }

    // Step 2: AUTH on the same connection
    let auth_cmd = format!("AUTH {}\n", hash);
    stream
        .write_all(auth_cmd.as_bytes())
        .map_err(|e| format!("Failed to send AUTH: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read AUTH response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("PPC auth failed: {} ({})", msg, code));
    }

    // Step 3: PPCPATH on the same connection
    stream
        .write_all(b"PPCPATH\n")
        .map_err(|e| format!("Failed to send PPCPATH: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read PPCPATH response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("Failed to query PPC path: {} ({})", msg, code));
    }
    let path = parse_payload(&response)
        .unwrap_or_default()
        .trim()
        .to_string();
    if path.is_empty() {
        return Err("Cannot parse PPC path from response".to_string());
    }

    let path_buf = std::path::PathBuf::from(path);
    log::info!("PPC base path resolved: {}", path_buf.display());

    // Cache for subsequent lookups and clear any failure cooldown
    *PPC_BASE_PATH_CACHE.lock().unwrap() = Some(path_buf.clone());
    *PPC_FAIL_COOLDOWN.lock().unwrap() = None;

    Ok(path_buf)
}

/// Resolve the application data directory for For_Your_File.
///
/// All mutable application data is stored under:
///   `%APPDATA%/wang.station/app/For_Your_File`
///
/// PPC remains available as a communication service, but it does not control
/// this application's local paths.
pub fn resolve_data_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let app_data_dir = app_data_dir
        .parent()
        .ok_or_else(|| "Failed to resolve the roaming app data directory".to_string())?
        .join("wang.station")
        .join("app")
        .join("For_Your_File");

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    log::info!(
        "Data directory resolved to wang.station path: {}",
        app_data_dir.display()
    );
    Ok(app_data_dir)
}

// ---------------------------------------------------------------------------
// PPC launcher
// ---------------------------------------------------------------------------

/// Find PPC in the standard 64-bit or 32-bit installation directory.
#[cfg(windows)]
fn find_installed_ppc() -> Option<std::path::PathBuf> {
    let candidates = [
        std::path::PathBuf::from(r"C:\Program Files\PPC\ppc.exe"),
        std::path::PathBuf::from(r"C:\Program Files (x86)\PPC\ppc.exe"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

/// Launch an installed PPC application with administrator privileges through UAC.
#[cfg(windows)]
fn launch_installed_ppc() -> Result<std::process::Child, String> {
    let executable = find_installed_ppc()
        .ok_or_else(|| "PPC executable was not found in standard install paths".to_string())?;
    let executable = executable.to_string_lossy().replace('"', "'");

    log::info!(
        "Launching installed PPC with administrator privileges: {}",
        executable
    );
    let script = format!(
        "Start-Process -FilePath '{}' -Verb RunAs -WorkingDirectory (Split-Path -Parent '{}')",
        executable, executable
    );

    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .map_err(|e| format!("Failed to launch installed PPC ({}): {}", executable, e))
}

/// Launch PPC in a visible administrator terminal through UAC.
/// If no installed executable exists, use the `ppc` command from PATH.
#[cfg(windows)]
fn launch_ppc_in_terminal() -> Result<std::process::Child, String> {
    let executable = find_installed_ppc()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ppc".to_string());

    log::info!(
        "Launching PPC in an elevated visible terminal: {}",
        executable
    );

    // Start-Process -Verb RunAs triggers the Windows UAC prompt and starts
    // the terminal, plus PPC, with administrator privileges.
    let command_line = format!(r#"/K "{}""#, executable).replace('"', "''");
    let script = format!(
        "Start-Process -FilePath 'cmd.exe' -ArgumentList '{}' -Verb RunAs -Wait",
        command_line,
    );

    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to start elevated PPC terminal ({}): {}",
                executable, e
            )
        })
}

/// Wait for a TCP port to become connectable, polling at a fixed interval.
/// Returns Ok(()) if the port becomes available within `timeout_secs`.
fn wait_for_port(host: &str, port: u16, timeout_secs: u64) -> bool {
    let addr = format!("{}:{}", host, port);
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

    while std::time::Instant::now() < deadline {
        if let Ok(stream) = std::net::TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:9527".parse().unwrap()),
            Duration::from_millis(500),
        ) {
            drop(stream);
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    false
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Connect to PPC with auto-launch: retry ping three times → (launch if down) →
/// version check → register → authenticate.
/// If PPC cannot be reached after launching, a warning dialog is shown.
#[tauri::command]
pub fn ppc_connect_auto(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, PpcState>,
) -> Result<PpcSession, String> {
    // Step 1: Verify the connection three times before diagnosing PPC as down.
    let mut running = false;
    for attempt in 1..=3 {
        if ping_ppc().unwrap_or(false) {
            running = true;
            break;
        }
        log::warn!("PPC connection attempt {}/3 failed", attempt);
        if attempt < 3 {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    if !running {
        log::info!("PPC is not connected after three attempts; checking startup...");

        // First try the installed application. It is the normal launch path
        // and preserves its own working directory and startup configuration.
        #[cfg(windows)]
        {
            let installed_started = launch_installed_ppc().is_ok();
            if installed_started && wait_for_port(PPC_HOST, PPC_PORT, 10) {
                log::info!("Installed PPC is reachable, continuing with connection...");
            } else {
                log::warn!("Installed PPC did not become reachable; falling back to terminal...");
                match launch_ppc_in_terminal() {
                    Ok(_child) if wait_for_port(PPC_HOST, PPC_PORT, 10) => {
                        log::info!("PPC terminal is reachable, continuing with connection...");
                    }
                    Ok(_) => {
                        let err_msg =
                            "PPC did not become reachable after installed app and terminal launch"
                                .to_string();
                        log::error!("{}", err_msg);
                        let mut session = state.session.lock().map_err(|e| e.to_string())?;
                        session.connected = false;
                        session.status_message = err_msg.clone();
                        session.last_error_code = Some("0x10017".to_string());
                        show_ppc_warning_dialog(&app_handle, &err_msg);
                        return Err(err_msg);
                    }
                    Err(e) => {
                        log::error!("Failed to launch PPC in terminal: {}", e);
                        let mut session = state.session.lock().map_err(|e| e.to_string())?;
                        session.connected = false;
                        session.status_message = e.clone();
                        session.last_error_code = Some("0x10017".to_string());
                        show_ppc_warning_dialog(&app_handle, &e);
                        return Err(e);
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            let err_msg =
                "PPC not running and auto-launch is only supported on Windows".to_string();
            {
                let mut session = state.session.lock().map_err(|e| e.to_string())?;
                session.connected = false;
                session.status_message = err_msg.clone();
                session.last_error_code = Some("0x10017".to_string());
            }
            show_ppc_warning_dialog(
                &app_handle,
                "Failed to connect to PPC. Auto-launch is only supported on Windows.",
            );
            return Err(err_msg);
        }
    }

    // Step 2: Check PPC version
    let ppc_version = match check_ppc_version() {
        Ok(v) => v,
        Err(e) => {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = e.clone();
            session.last_error_code = Some("0x10019".to_string());
            return Err(e);
        }
    };

    // Step 3: Register
    let app_hash = match register_with_ppc() {
        Ok(h) => h,
        Err(e) => {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = e.clone();
            session.last_error_code = Some("0x10005".to_string());
            return Err(e);
        }
    };

    // Step 4: Authenticate
    if let Err(e) = authenticate_with_ppc(&app_hash) {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        session.connected = false;
        session.app_hash = Some(app_hash);
        session.ppc_version = Some(ppc_version);
        session.status_message = e.clone();
        session.last_error_code = Some("0x10018".to_string());
        return Err(e);
    }

    // Success: update session state
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session.connected = true;
    session.app_hash = Some(app_hash);
    session.ppc_version = Some(ppc_version.clone());
    session.status_message = format!("Connected to PPC v{}", ppc_version);
    session.last_error_code = None;

    // Clear the failure cooldown now that PPC is confirmed reachable
    *PPC_FAIL_COOLDOWN.lock().unwrap() = None;

    Ok(session.clone())
}

/// Show a modal warning dialog when PPC cannot be connected.
fn show_ppc_warning_dialog(app_handle: &tauri::AppHandle, message: &str) {
    use tauri_plugin_dialog::DialogExt;

    log::warn!("Showing PPC warning dialog: {}", message);

    app_handle
        .dialog()
        .message(message)
        .title("PPC Connection Warning")
        .show(|_| {
            log::info!("PPC warning dialog dismissed");
        });
}

/// Get current PPC session status (without connecting).
#[tauri::command]
pub fn ppc_status(state: tauri::State<'_, PpcState>) -> PpcSession {
    let session = state.session.lock().unwrap_or_else(|e| {
        log::error!("PPC state lock poisoned: {}", e);
        std::process::abort();
    });
    session.clone()
}

/// Send a raw command to PPC. Requires an active session (will re-authenticate
/// if needed, since PPC requires AUTH on every new TCP connection).
#[tauri::command]
pub fn ppc_send_command(
    state: tauri::State<'_, PpcState>,
    command: String,
) -> Result<String, String> {
    let (hash, connected) = {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        (session.app_hash.clone(), session.connected)
    };

    if !connected {
        return Err("PPC not connected, call ppc_connect first".to_string());
    }

    let hash = hash.ok_or("No auth hash, register first")?;

    // PPC requires AUTH on every new TCP connection, so we send AUTH first,
    // then the actual command. Both must go through the same TCP connection.
    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(TIMEOUT_SECS),
    )
    .map_err(|e| format!("Failed to connect to PPC: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    // Send AUTH
    let auth_cmd = format!("AUTH {}\n", hash);
    stream
        .write_all(auth_cmd.as_bytes())
        .map_err(|e| format!("Failed to send AUTH: {}", e))?;

    let mut buf = [0u8; 4096];
    let _auth_n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read AUTH response: {}", e))?;

    // Verify AUTH succeeded before sending the actual command
    let auth_response = String::from_utf8(buf[.._auth_n].to_vec()).unwrap_or_default();
    let auth_code = parse_status_code(&auth_response).unwrap_or_default();
    if !is_success_code(&auth_code) {
        let msg = map_ppc_code(&auth_code);
        // Update session state to reflect disconnection
        {
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = format!("PPC auth expired: {} ({})", msg, auth_code);
            session.last_error_code = Some(auth_code.clone());
        }
        return Err(format!(
            "PPC authentication failed: {} ({})",
            msg, auth_code
        ));
    }

    // Send the actual command
    let cmd_bytes = format!("{}\n", command);
    stream
        .write_all(cmd_bytes.as_bytes())
        .map_err(|e| format!("Failed to send command: {}", e))?;

    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let response =
        String::from_utf8(buf[..n].to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))?;

    // Parse status code and map to readable message
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        log::warn!(
            "PPC command '{}' returned error: {} ({})",
            command,
            msg,
            code
        );
    }

    Ok(response)
}

/// Disconnect from PPC (clears the session state).
#[tauri::command]
pub fn ppc_disconnect(state: tauri::State<'_, PpcState>) -> Result<(), String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session.connected = false;
    session.status_message = "Disconnected".to_string();
    Ok(())
}

/// Get the list of PPC error codes and their descriptions.
/// Useful for the frontend to display error reference.
#[tauri::command]
pub fn ppc_error_codes() -> Vec<(String, String)> {
    vec![
        ("0x00000".to_string(), "Success".to_string()),
        (
            "0x00001".to_string(),
            "App registered successfully".to_string(),
        ),
        (
            "0x00002".to_string(),
            "App updated successfully".to_string(),
        ),
        ("0x00003".to_string(), "DLL loaded successfully".to_string()),
        (
            "0x00004".to_string(),
            "Command executed successfully".to_string(),
        ),
        ("0x10000".to_string(), "Unknown error".to_string()),
        ("0x10001".to_string(), "Unsupported command".to_string()),
        ("0x10002".to_string(), "Resource not found".to_string()),
        (
            "0x10003".to_string(),
            "Signature verification failed".to_string(),
        ),
        ("0x10004".to_string(), "Hash mismatch".to_string()),
        ("0x10005".to_string(), "Registration failed".to_string()),
        ("0x10006".to_string(), "Update failed".to_string()),
        (
            "0x10007".to_string(),
            "App not registered or not authenticated".to_string(),
        ),
        ("0x10008".to_string(), "Invalid parameters".to_string()),
        ("0x10009".to_string(), "File not found".to_string()),
        ("0x10010".to_string(), "Permission denied".to_string()),
        ("0x10011".to_string(), "DLL not found".to_string()),
        ("0x10012".to_string(), "DLL load failed".to_string()),
        ("0x10013".to_string(), "Config parse failed".to_string()),
        ("0x10014".to_string(), "Database error".to_string()),
        ("0x10015".to_string(), "Connection failed".to_string()),
        ("0x10016".to_string(), "Timeout".to_string()),
        ("0x10017".to_string(), "PPC server not running".to_string()),
        (
            "0x10018".to_string(),
            "PPC authentication failed".to_string(),
        ),
        (
            "0x10019".to_string(),
            "PPC version not supported".to_string(),
        ),
        ("0x10020".to_string(), "PPC response invalid".to_string()),
        ("0x10021".to_string(), "Window pin failed".to_string()),
        ("0x10022".to_string(), "Window not found".to_string()),
        (
            "0x10023".to_string(),
            "Hotkey registration failed".to_string(),
        ),
        ("0x10024".to_string(), "Config save failed".to_string()),
        ("0x10025".to_string(), "Config load failed".to_string()),
        (
            "0x10026".to_string(),
            "State persistence failed".to_string(),
        ),
        (
            "0x10027".to_string(),
            "Administrator privileges required".to_string(),
        ),
        (
            "0x10028".to_string(),
            "Localization load failed".to_string(),
        ),
        (
            "0x10029".to_string(),
            "Single instance violation".to_string(),
        ),
        (
            "0x10030".to_string(),
            "Hook installation failed".to_string(),
        ),
        ("0x10031".to_string(), "Log write failed".to_string()),
        ("0x10032".to_string(), "DLL call failed".to_string()),
        ("0x20000".to_string(), "Waiting for signature".to_string()),
        ("0x20001".to_string(), "Waiting for app info".to_string()),
        ("0x20002".to_string(), "Processing".to_string()),
        ("0x20003".to_string(), "Initializing".to_string()),
        ("0x20004".to_string(), "Loading config".to_string()),
        ("0x20005".to_string(), "Executing command".to_string()),
        ("0x20006".to_string(), "Connecting".to_string()),
        ("0x20007".to_string(), "Registering".to_string()),
        ("0x20008".to_string(), "Authenticating".to_string()),
        ("0x20009".to_string(), "Reconnecting".to_string()),
        (
            "0x30000".to_string(),
            "Deprecated command warning".to_string(),
        ),
        ("0x30001".to_string(), "Old version warning".to_string()),
        (
            "0x30002".to_string(),
            "High memory usage warning".to_string(),
        ),
        ("0x30003".to_string(), "PPC reconnect warning".to_string()),
        (
            "0x30004".to_string(),
            "Fallback behavior warning".to_string(),
        ),
        (
            "0x30005".to_string(),
            "Language fallback warning".to_string(),
        ),
    ]
}
