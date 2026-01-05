; -----------------------------
; hsemulate Windows Installer
; -----------------------------

!define APP_NAME "hsemulate"
!define APP_EXE "hsemulate.exe"
!define APP_VERSION "0.1.0"
!define INSTALL_DIR "$PROGRAMFILES\hsemulate"

OutFile "hsemulate-${APP_VERSION}-windows-installer.exe"
InstallDir "${INSTALL_DIR}"
RequestExecutionLevel admin

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

; -----------------------------
; Install
; -----------------------------
Section "Install"
  SetOutPath "$INSTDIR"

  ; Copy release binary
  File "target\release\${APP_EXE}"

  ; Add to PATH (machine-wide)
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  StrCpy $1 "$0;$INSTDIR"
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$1"
  SendMessage HWND_BROADCAST WM_SETTINGCHANGE 0 "STR:Environment" /TIMEOUT=5000

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Add uninstall entry
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "MIT"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" "$INSTDIR\Uninstall.exe"
SectionEnd

; -----------------------------
; Uninstall
; -----------------------------
Section "Uninstall"
  ; Remove files
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  ; Remove from PATH
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  ${EnvVarUpdate} $0 "Path" "R" "HKLM" "$INSTDIR"
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$0"
  SendMessage HWND_BROADCAST WM_SETTINGCHANGE 0 "STR:Environment" /TIMEOUT=5000

  ; Remove uninstall registry entry
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd
