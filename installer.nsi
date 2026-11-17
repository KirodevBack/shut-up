Unicode true
!include "MUI2.nsh"

!define APP "Shut up"
!define EXE "shut-up.exe"
!ifndef VERSION
  !define VERSION "1.0.0"
!endif
!define UNINST "Software\Microsoft\Windows\CurrentVersion\Uninstall\shut-up"
!define RUNKEY "Software\Microsoft\Windows\CurrentVersion\Run"

Name "${APP}"
OutFile "target\shut-up-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\shut-up"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "${APP} ${VERSION}"

!define MUI_ICON "assets\shut-up.ico"
!define MUI_UNICON "assets\shut-up.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Lancer ${APP}"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "French"

Section "Install"
  nsExec::Exec 'taskkill /im ${EXE}'
  Sleep 800
  SetOutPath "$INSTDIR"
  File "target\release\${EXE}"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "${RUNKEY}" "shut-up" '"$INSTDIR\${EXE}"'
  CreateShortcut "$SMPROGRAMS\${APP}.lnk" "$INSTDIR\${EXE}"
  WriteRegStr HKCU "${UNINST}" "DisplayName" "${APP}"
  WriteRegStr HKCU "${UNINST}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINST}" "Publisher" "shut-up"
  WriteRegStr HKCU "${UNINST}" "DisplayIcon" "$INSTDIR\${EXE}"
  WriteRegStr HKCU "${UNINST}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKCU "${UNINST}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST}" "NoRepair" 1
  WriteRegDWORD HKCU "${UNINST}" "EstimatedSize" 200
SectionEnd

Section "Uninstall"
  nsExec::Exec 'taskkill /im ${EXE}'
  Sleep 1500
  DeleteRegValue HKCU "${RUNKEY}" "shut-up"
  DeleteRegKey HKCU "${UNINST}"
  Delete "$SMPROGRAMS\${APP}.lnk"
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  RMDir /r "$APPDATA\shut-up"
SectionEnd
