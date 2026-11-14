// fyf-launcher — For Your File bootstrapper.
//
// Since v0.0.5 the PPC daemon is started by this launcher (Go) instead of the
// Tauri/Rust process. Order of operations:
//
//  1. Probe 127.0.0.1:9527 (PPC's local TCP port).
//  2. If PPC is not running, start %APPDATA%\wang.station\ppc.exe (or
//     ppc32.exe) as a fully detached, window-less process.
//  3. Wait until the port accepts connections (UAC elevation may take a while).
//  4. Start the For Your File executable next to this launcher and forward
//     all command-line arguments to it.
//
// Errors are reported through PPC's `WINDOW ERROR` command whenever PPC is
// reachable. Only when PPC itself cannot be reached does the launcher fall back
// to a Win32 MessageBox — it is the bootstrap layer, so there is no PPC to ask.
//
// Build (Windows):
//
//	GOOS=windows GOARCH=amd64 CGO_ENABLED=0 \
//	  go build -trimpath -ldflags "-s -w -H windowsgui" -o fyf-launcher.exe .
//
// A helper script is provided in build.ps1.
package main

import (
	"errors"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"syscall"
	"time"
	"unsafe"
)

const (
	ppcHost = "127.0.0.1"
	ppcPort = "9527"

	// PPC performs a UAC elevation on startup; give it up to five minutes on
	// slow machines before giving up.
	ppcStartTimeout = 5 * time.Minute
	pollInterval    = time.Second
	dialTimeout     = 400 * time.Millisecond

	// Windows process creation flags.
	createNoWindow        = 0x08000000
	detachedProcess       = 0x00000008
	createNewProcessGroup = 0x00000200

	// MessageBoxW flags.
	mbOK        = 0x00000000
	mbIconError = 0x00000010
)

var (
	user32          = syscall.NewLazyDLL("user32.dll")
	procMessageBoxW = user32.NewProc("MessageBoxW")

	logFile *os.File
)

func main() {
	initLog()
	if logFile != nil {
		defer logFile.Close()
	}

	logf("Launcher arch: %s/%s", runtime.GOOS, runtime.GOARCH)
	ppcExe, err := findPPC()
	if err != nil {
		showError(err.Error())
		return
	}
	logf("PPC executable: %s", ppcExe)

	if !portOpen() {
		logf("PPC is not running, starting it detached...")
		if err := startPPC(ppcExe); err != nil {
			showError(tr("ppcStartFailed", "detail", err.Error()))
			return
		}
		if !waitForPPC(ppcStartTimeout) {
			showError(tr(
				"ppcTimeout",
				"seconds", fmt.Sprintf("%d", int(ppcStartTimeout.Seconds())),
				"port", ppcPort,
			))
			return
		}
		logf("PPC is ready")
	} else {
		logf("PPC already running")
	}

	appExe, err := findApp()
	if err != nil {
		showError(err.Error())
		return
	}
	logf("Starting app: %s", appExe)
	if err := startApp(appExe, os.Args[1:]); err != nil {
		showError(tr("appStartFailed", "detail", err.Error()))
		return
	}
	logf("Launcher done")
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

func initLog() {
	appData := os.Getenv("APPDATA")
	if appData == "" {
		return
	}
	dir := filepath.Join(appData, "wang.station", "logs")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return
	}
	name := fmt.Sprintf("launcher-%s.log", time.Now().Format("20060102-150405"))
	f, err := os.OpenFile(filepath.Join(dir, name), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o644)
	if err == nil {
		logFile = f
	}
}

func logf(format string, args ...any) {
	line := fmt.Sprintf("%s %s\n", time.Now().Format("2006-01-02 15:04:05"), fmt.Sprintf(format, args...))
	if logFile != nil {
		_, _ = logFile.WriteString(line)
	}
}

// ---------------------------------------------------------------------------
// PPC discovery / startup
// ---------------------------------------------------------------------------

func findPPC() (string, error) {
	appData := os.Getenv("APPDATA")
	if appData == "" {
		return "", errors.New(tr("noAppData"))
	}
	base := filepath.Join(appData, "wang.station")

	// Match the PPC binary to this launcher's architecture: a 64-bit launcher
	// starts ppc.exe (x86-64), a 32-bit launcher starts ppc32.exe (x86).
	// The other binary is kept as a fallback so a partial installation still works.
	names := []string{"ppc.exe", "ppc32.exe"}
	if runtime.GOARCH == "386" {
		names = []string{"ppc32.exe", "ppc.exe"}
	}
	for _, name := range names {
		candidate := filepath.Join(base, name)
		if info, err := os.Stat(candidate); err == nil && !info.IsDir() {
			return candidate, nil
		}
	}
	return "", errors.New(tr("ppcNotFound", "path", filepath.Join(base, names[0])))
}

func portOpen() bool {
	conn, err := net.DialTimeout("tcp", net.JoinHostPort(ppcHost, ppcPort), dialTimeout)
	if err != nil {
		return false
	}
	_ = conn.Close()
	return true
}

func startPPC(exe string) error {
	cmd := exec.Command(exe)
	cmd.Dir = filepath.Dir(exe)
	cmd.SysProcAttr = &syscall.SysProcAttr{
		HideWindow:    true,
		CreationFlags: createNoWindow | detachedProcess | createNewProcessGroup,
	}
	if err := cmd.Start(); err != nil {
		return err
	}
	// Release the handle: PPC keeps running independently of this launcher.
	return cmd.Process.Release()
}

func waitForPPC(timeout time.Duration) bool {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if portOpen() {
			return true
		}
		time.Sleep(pollInterval)
	}
	return false
}

// ---------------------------------------------------------------------------
// Application discovery / startup
// ---------------------------------------------------------------------------

func findApp() (string, error) {
	exe, err := os.Executable()
	if err != nil {
		return "", errors.New(tr("noLauncherPath", "detail", err.Error()))
	}
	exeDir := filepath.Dir(exe)

	names := []string{
		"For Your File.exe",
		"For_Your_File.exe",
		"LNK File Management Center.exe",
		"lnk-file-management-center.exe",
		"app.exe",
	}
	dirs := []string{exeDir, filepath.Dir(exeDir)}
	for _, dir := range dirs {
		for _, name := range names {
			candidate := filepath.Join(dir, name)
			if info, err := os.Stat(candidate); err == nil && !info.IsDir() {
				return candidate, nil
			}
		}
	}
	return "", errors.New(tr("noAppFound"))
}

func startApp(exe string, args []string) error {
	cmd := exec.Command(exe, args...)
	cmd.Dir = filepath.Dir(exe)
	if err := cmd.Start(); err != nil {
		return err
	}
	return cmd.Process.Release()
}

// ---------------------------------------------------------------------------
// Error reporting
// ---------------------------------------------------------------------------

// showError reports through PPC when possible, otherwise a native MessageBox.
func showError(message string) {
	logf("ERROR: %s", message)
	if ppcWindowError(message) {
		return
	}
	title, _ := syscall.UTF16PtrFromString(tr("messageTitle"))
	text, _ := syscall.UTF16PtrFromString(message)
	procMessageBoxW.Call(
		0,
		uintptr(unsafe.Pointer(title)),
		uintptr(unsafe.Pointer(text)),
		uintptr(mbOK|mbIconError),
	)
}

func ppcWindowError(message string) bool {
	conn, err := net.DialTimeout("tcp", net.JoinHostPort(ppcHost, ppcPort), time.Second)
	if err != nil {
		return false
	}
	defer conn.Close()
	_ = conn.SetWriteDeadline(time.Now().Add(2 * time.Second))
	_, err = conn.Write([]byte("WINDOW ERROR " + message + "\n"))
	return err == nil
}
