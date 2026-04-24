; RustyTeams NSIS installer.
;
; Build first with `make package` (which stages everything into dist/rustyteams/)
; then run `makensis installer.nsi` (or `make installer`).

!define APP_NAME       "RustyTeams"
!define APP_VER        "0.1.0"
!define APP_PUB        "alexieff.io"
!define APP_EXE        "rustyteams.exe"
!define APP_REGKEY     "Software\${APP_NAME}"
!define APP_UNINSTKEY  "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
!define APP_AUMID      "io.alexieff.rustyteams"

Name            "${APP_NAME}"
OutFile         "dist\${APP_NAME}-Setup-${APP_VER}.exe"
InstallDir      "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "${APP_REGKEY}" "InstallDir"
RequestExecutionLevel admin
Unicode         true
SetCompressor   /SOLID lzma

VIProductVersion    "${APP_VER}.0"
VIAddVersionKey     "ProductName"     "${APP_NAME}"
VIAddVersionKey     "CompanyName"     "${APP_PUB}"
VIAddVersionKey     "FileDescription" "Lightweight Microsoft Teams desktop wrapper"
VIAddVersionKey     "FileVersion"     "${APP_VER}"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
    SetOutPath "$INSTDIR"
    File /r "dist\rustyteams\*.*"

    WriteRegStr   HKLM "${APP_REGKEY}"    "InstallDir"      "$INSTDIR"
    WriteRegStr   HKLM "${APP_REGKEY}"    "AppUserModelID"  "${APP_AUMID}"

    WriteRegStr   HKLM "${APP_UNINSTKEY}" "DisplayName"     "${APP_NAME}"
    WriteRegStr   HKLM "${APP_UNINSTKEY}" "DisplayVersion"  "${APP_VER}"
    WriteRegStr   HKLM "${APP_UNINSTKEY}" "Publisher"       "${APP_PUB}"
    WriteRegStr   HKLM "${APP_UNINSTKEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr   HKLM "${APP_UNINSTKEY}" "DisplayIcon"     "$INSTDIR\${APP_EXE}"
    WriteRegStr   HKLM "${APP_UNINSTKEY}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegDWORD HKLM "${APP_UNINSTKEY}" "NoModify"        1
    WriteRegDWORD HKLM "${APP_UNINSTKEY}" "NoRepair"        1

    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortCut  "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"  "$INSTDIR\${APP_EXE}" "" "$INSTDIR\${APP_EXE}" 0
    CreateShortCut  "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"    "$INSTDIR\uninstall.exe"

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
    RMDir  "$SMPROGRAMS\${APP_NAME}"

    RMDir /r "$INSTDIR"

    DeleteRegKey HKLM "${APP_REGKEY}"
    DeleteRegKey HKLM "${APP_UNINSTKEY}"
SectionEnd
