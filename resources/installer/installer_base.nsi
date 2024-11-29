;-------------------------------------------------------------------------------
; Includes
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinVer.nsh"
!include "x64.nsh"

;-------------------------------------------------------------------------------
; Constants
!define PRODUCT_NAME "My Plugin"
!define PRODUCT_DESCRIPTION "My VST Plugin"
!define COPYRIGHT "Copyright (c) 2024 Nanashi."
!define PRODUCT_VERSION "{version}.0"
!define SETUP_VERSION "{version}.0"

;-------------------------------------------------------------------------------
; Attributes
Name "my_plugin"
OutFile "build/my_plugin-{version}-windows-setup.exe"
InstallDir "$PROGRAMFILES64\Common Files\VST3\my_plugin.vst3"
RequestExecutionLevel admin ; user|highest|admin

;-------------------------------------------------------------------------------
; Version Info
VIProductVersion "${PRODUCT_VERSION}"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "ProductVersion" "${PRODUCT_VERSION}"
VIAddVersionKey "FileDescription" "${PRODUCT_DESCRIPTION}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${SETUP_VERSION}"

;-------------------------------------------------------------------------------
; Modern UI Appearance
!define MUI_ICON "resources\installer\my_plugin.ico"

;-------------------------------------------------------------------------------
; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

;-------------------------------------------------------------------------------
; Uninstaller Pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_DIRECTORY
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;-------------------------------------------------------------------------------
; Languages
!insertmacro MUI_LANGUAGE "Japanese"

;-------------------------------------------------------------------------------
; Installer Sections
Section "my_plugin" Vvvst
	SetOutPath "$INSTDIR"
  File "resources\installer\my_plugin.ico"
  File "resources\installer\desktop.ini"
  File /r "build\release\bin\vvvst.vst3\"
  System::Call "shlwapi::PathMakeSystemFolder(t '$INSTDIR') i."

  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\my_plugin" \
                   "DisplayName" "my_plugin"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\my_plugin" \
                   "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
SectionEnd

;-------------------------------------------------------------------------------
; Uninstaller Sections
Section "Uninstall"
	RMDir /r "$INSTDIR"
  Delete "$INSTDIR\Uninstall.exe"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\my_plugin"
SectionEnd
