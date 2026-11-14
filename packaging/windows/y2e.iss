; Inno Setup script for the Windows installer. Built by the release workflow:
;   iscc /DVersion=0.2.0 /DOutName=y2e-v0.2.0-windows-x86_64-setup packaging\windows\y2e.iss
; Paths are relative to this file. The installer needs no administrator rights.

#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef OutName
  #define OutName "y2e-setup"
#endif

[Setup]
AppId={{C95A131B-D3A3-45E8-8173-6CBC5427E5A8}
AppName=YouTube to Everything
AppVersion={#Version}
AppPublisher=MedCy1
AppPublisherURL=https://github.com/MedCy1/Youtube-To-Everything
AppSupportURL=https://github.com/MedCy1/Youtube-To-Everything/issues
DefaultDirName={autopf}\y2e
DefaultGroupName=YouTube to Everything
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\y2e-gui.exe
SetupIconFile=..\y2e.ico
OutputDir=..\..\out
OutputBaseFilename={#OutName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Files]
Source: "..\..\target\dist\y2e-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\YouTube to Everything"; Filename: "{app}\y2e-gui.exe"

[Run]
Filename: "{app}\y2e-gui.exe"; Description: "Start YouTube to Everything"; Flags: nowait postinstall skipifsilent
