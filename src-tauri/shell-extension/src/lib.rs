//! Windows Shell Extension for LNK File Management Center
//!
//! This shell extension adds "Add to File Management Center" to Windows Explorer context menu

use std::ffi::c_void;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const MENU_TEXT: &str = "Add to File Management Center";
const PIPE_NAME: &str = "\\\\.\\pipe\\LnkManagementCenter";

/// Shell extension class implementing IContextMenu and IShellExtInit
#[derive(Debug)]
struct LnkShellExtension {
    ref_count: u32,
    selected_files: Vec<String>,
}

impl LnkShellExtension {
    fn new() -> Self {
        Self {
            ref_count: 1,
            selected_files: Vec::new(),
        }
    }

    /// Send file path to main application via named pipe
    fn send_to_application(&self, file_path: &str) -> Result<()> {
        // Create pipe name with full path
        let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME).encode_wide().chain(Some(0)).collect();

        // Try to connect to the named pipe
        let pipe_handle = unsafe {
            CreateFileW(
                PCWSTR(pipe_name.as_ptr()),
                GENERIC_WRITE.0,
                FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE::default(),
            )
        };

        if pipe_handle.is_invalid() {
            log::error!("Failed to connect to named pipe");
            return Err(Error::from_win32());
        }

        // Send file path to application
        let message = format!("ADD:{}", file_path);
        let bytes = message.as_bytes();

        unsafe {
            let mut bytes_written: u32 = 0;
            let result = WriteFile(
                pipe_handle,
                Some(bytes.as_ptr() as *const _),
                bytes.len() as u32,
                &mut bytes_written,
                None,
            );

            if result.is_ok() && bytes_written == bytes.len() as u32 {
                log::info!("Successfully sent file path: {}", file_path);
            } else {
                log::error!("Failed to write to pipe");
            }

            let _ = CloseHandle(pipe_handle);
        }

        Ok(())
    }
}

#[implement(IContextMenu, IShellExtInit)]
impl LnkShellExtension {
    fn Initialize(
        &self,
        pidl_folder: *const ITEMIDLIST,
        pidl_data: *mut *const ITEMIDLIST,
        _hkey: HKEY,
    ) -> windows::core::Result<()> {
        // Note: We need to modify selected_files, but in a COM interface
        // we can't easily do that without interior mutability
        // For now, we'll use a simpler approach

        if pidl_data.is_null() {
            return Ok(());
        }

        log::info!("Shell extension initialized");

        Ok(())
    }

    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        index_menu: u32,
        id_cmd_first: u32,
        _id_cmd_last: u32,
        _u_flags: CMF,
    ) -> windows::core::Result<i32> {
        // Insert menu item
        let menu_text: Vec<u16> = OsStr::new(MENU_TEXT)
            .encode_wide()
            .chain(Some(0))
            .collect();

        unsafe {
            let result = InsertMenuW(
                hmenu,
                index_menu as i32,
                MF_STRING | MF_BYPOSITION,
                usize::from(id_cmd_first),
                PCWSTR(menu_text.as_ptr()),
            );

            if result.as_bool() {
                log::info!("Context menu item added successfully");
                return Ok(1); // Return number of items added
            } else {
                log::error!("Failed to insert menu item");
                return Err(Error::from_win32());
            }
        }
    }

    fn InvokeCommand(&self, picmi: *const CMINVOKECOMMANDINFO) -> windows::core::Result<()> {
        if picmi.is_null() {
            return Err(Error::from_win32());
        }

        let info = unsafe { &*picmi };

        // Check if this is our command (verb = 0)
        // For simplicity, we'll just handle the first command

        log::info!("Command invoked from context menu");

        // In a real implementation, we would:
        // 1. Get the selected files from the initialize call
        // 2. Send them to the main application via named pipe

        // For now, show a message
        unsafe {
            MessageBoxW(
                None,
                windows::core::w!("File will be added to LNK File Management Center"),
                windows::core::w!("LNK Shell Extension"),
                MB_OK,
            );
        }

        Ok(())
    }

    fn GetCommandString(
        &self,
        id_cmd: usize,
        _u_flags: GCS,
        _p_reserved: *mut u32,
        psz_name: PSTR,
        _cch_max: i32,
    ) -> windows::core::Result<()> {
        // Only handle our single command (ID 0)
        if id_cmd != 0 {
            return Err(Error::from_win32());
        }

        // Return the help text for the menu item
        let help_text = "Add the selected file to LNK File Management Center";
        unsafe {
            std::ptr::copy_nonoverlapping(
                help_text.as_ptr(),
                psz_name.as_ptr(),
                help_text.len(),
            );
            *psz_name.as_ptr().add(help_text.len()) = 0;
        }

        Ok(())
    }
}

// COM DLL exports
#[no_mangle]
extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // CLSID for our shell extension
    const CLSID_LnkShellExtension: GUID = GUID::from_u128(0x12345678_1234_1234_1234_123456789ABC);

    unsafe {
        if ppv.is_null() || rclsid.is_null() || riid.is_null() {
            return ERROR_INVALID_PARAMETER.into();
        }

        if *rclsid != CLSID_LnkShellExtension {
            *ppv = ptr::null_mut();
            return CLASS_E_CLASSNOTAVAILABLE.into();
        }

        // Create instance - simplified for now
        // In production, we'd implement a proper class factory
        let instance = LnkShellExtension::new();
        let instance: IContextMenu = instance.into();

        match instance.QueryInterface(*riid, ppv) {
            Ok(()) => S_OK.into(),
            Err(e) => e.into(),
        }
    }
}

#[no_mangle]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    // For simplicity, always return S_FALSE (cannot unload)
    S_FALSE.into()
}

#[no_mangle]
extern "system" fn DllRegisterServer() -> HRESULT {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Path to registration script
    let script_path = std::env::current_exe()
        .and_then(|exe| {
            exe.parent()
                .map(|p| p.join("install-context-menu.ps1"))
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Parent directory not found"))
        });

    if let Ok(script_path) = script_path {
        // Execute PowerShell registration script
        let result = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-File", &script_path.to_string_lossy()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                log::info!("Shell extension registered successfully");
                return S_OK.into();
            } else {
                log::error!("Registration failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }

    E_FAIL.into()
}

#[no_mangle]
extern "system" fn DllUnregisterServer() -> HRESULT {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Path to unregistration script
    let script_path = std::env::current_exe()
        .and_then(|exe| {
            exe.parent()
                .map(|p| p.join("uninstall-context-menu.ps1"))
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Parent directory not found"))
        });

    if let Ok(script_path) = script_path {
        // Execute PowerShell unregistration script
        let result = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-File", &script_path.to_string_lossy()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                log::info!("Shell extension unregistered successfully");
                return S_OK.into();
            } else {
                log::error!("Unregistration failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }

    E_FAIL.into()
}

// DllMain equivalent
#[no_mangle]
extern "system" fn DllMain(_hinst: HINSTANCE, _reason: u32, _reserved: *mut ()) -> BOOL {
    TRUE.into()
}