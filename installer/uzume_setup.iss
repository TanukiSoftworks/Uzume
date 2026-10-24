; Inno Setup Script for Uzume
; Tanuki Softworks

#define MyAppName "Uzume"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Tanuki Softworks"
#define MyAppURL "https://uzume.tanukisoftworks.com"
#define MyAppRepoURL "https://github.com/TanukiSoftworks/Uzume"
#define MyAppExeName "uzume.exe"

[Setup]
; Unique GUID for Uzume
AppId={{8B190204-71B3-4A59-BF28-B410D2C8E7B2}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppRepoURL}/issues
AppUpdatesURL={#MyAppRepoURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\..\releases
OutputBaseFilename=Uzume-Setup-x64
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=force
CloseApplicationsFilter=*.exe
MinVersion=10.0
VersionInfoVersion=1.0.0.0
VersionInfoCompany=Tanuki Softworks
VersionInfoDescription=Uzume Setup (x64)
VersionInfoCopyright=Copyright (C) 2026 Tanuki Softworks under GNU 3.0 or newer
VersionInfoProductName=Uzume
VersionInfoProductVersion=1.0.0.0

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"; InfoBeforeFile: "notice_ja.txt"
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"; InfoBeforeFile: "notice_ko.txt"
Name: "chinesesimplified"; MessagesFile: "Languages\ChineseSimplified.isl"; InfoBeforeFile: "notice_zh_hans.txt"
Name: "chinesetraditional"; MessagesFile: "Languages\ChineseTraditional.isl"; InfoBeforeFile: "notice_zh_hant.txt"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startupicon"; Description: "Iniciar Uzume automáticamente con Windows"; GroupDescription: "Opciones de inicio:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startupicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
