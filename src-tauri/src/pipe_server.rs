//! Named pipe server for shell extension communication
//!
//! Receives file paths from shell extension and creates entries

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Manager};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Pipes::*;

const PIPE_NAME: &str = "\\\\.\\pipe\\LnkManagementCenter";
const BUFFER_SIZE: usize = 4096;

/// Start the named pipe server
pub fn start_pipe_server(app_handle: AppHandle) -> Result<(), String> {
    let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME)
        .encode_wide()
        .chain(Some(0))
        .collect();

    // Spawn background thread for pipe server
    thread::spawn(move || {
        loop {
            // Create named pipe
            let pipe_handle = unsafe {
                CreateNamedPipeW(
                    PCWSTR(pipe_name.as_ptr()),
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                    PIPE_UNLIMITED_INSTANCES,
                    BUFFER_SIZE as u32,
                    BUFFER_SIZE as u32,
                    0,
                    None,
                )
            };

            if pipe_handle.is_invalid() {
                log::error!("Failed to create named pipe: {:?}", Error::from_win32());
                thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }

            log::info!("Named pipe server waiting for connection...");

            // Wait for client connection
            let connect_result = unsafe { ConnectNamedPipe(pipe_handle, None) };
            if connect_result.is_err() {
                log::error!("Failed to connect to pipe client");
                unsafe { CloseHandle(pipe_handle).ok() };
                continue;
            }

            log::info!("Client connected to named pipe");

            // Read message from client
            let mut buffer = vec![0u8; BUFFER_SIZE];
            let mut bytes_read: u32 = 0;

            let read_result = unsafe {
                ReadFile(
                    pipe_handle,
                    Some(buffer.as_mut_ptr() as *mut _),
                    buffer.len() as u32,
                    &mut bytes_read,
                    None,
                )
            };

            if read_result.is_err() || bytes_read == 0 {
                log::error!("Failed to read from pipe");
                unsafe { CloseHandle(pipe_handle).ok() };
                continue;
            }

            // Parse message
            let message = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
            log::info!("Received message: {}", message);

            // Handle ADD command
            if message.starts_with("ADD:") {
                let file_path = message.trim_start_matches("ADD:").trim();

                // Emit event to frontend
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("add-file-from-context-menu", file_path);
                }
            }

            // Close pipe handle
            unsafe { CloseHandle(pipe_handle).ok() };
        }
    });

    log::info!("Named pipe server started on {}", PIPE_NAME);
    Ok(())
}