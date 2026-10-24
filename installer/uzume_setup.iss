; Inno Setup Script for Uzume
; Tanuki Softworks

#define MyAppName "Uzume"
#define MyAppVersion "1.1.0"
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
VersionInfoVersion=1.1.0.0
VersionInfoCompany=Tanuki Softworks
VersionInfoDescription=Uzume Setup (x64)
VersionInfoCopyright=Copyright (C) 2026 Tanuki Softworks under GNU 3.0 or newer
VersionInfoProductName=Uzume
VersionInfoProductVersion=1.1.0

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"; InfoBeforeFile: "notice_ja.txt"
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"; InfoBeforeFile: "notice_ko.txt"
Name: "chinesesimplified"; MessagesFile: "Languages\ChineseSimplified.isl"; InfoBeforeFile: "notice_zh_hans.txt"
Name: "chinesetraditional"; MessagesFile: "Languages\ChineseTraditional.isl"; InfoBeforeFile: "notice_zh_hant.txt"

[CustomMessages]
spanish.StartupTask=Iniciar Uzume automáticamente con Windows
spanish.StartupGroup=Opciones de inicio:
spanish.UpdateAppTitle=Actualizar %1
spanish.ButtonUpdate=Actualizar
spanish.UpdateReadyMsg=El asistente está listo para actualizar %1 a la versión %2 en su equipo.
spanish.UpdateFinishedHeading=Actualización de %1 completada
spanish.UpdateFinishedMsg=%1 se ha actualizado correctamente en su equipo.%n%nHaga clic en Finalizar para salir del asistente.

english.StartupTask=Start Uzume automatically with Windows
english.StartupGroup=Startup options:
english.UpdateAppTitle=Update %1
english.ButtonUpdate=Update
english.UpdateReadyMsg=Setup is ready to update %1 to version %2 on your computer.
english.UpdateFinishedHeading=%1 Update Complete
english.UpdateFinishedMsg=%1 has been updated on your computer.%n%nClick Finish to exit Setup.

japanese.StartupTask=Windows の起動時に Uzume を自動的に開始する
japanese.StartupGroup=スタートアップ オプション:
japanese.UpdateAppTitle=%1 の更新
japanese.ButtonUpdate=更新
japanese.UpdateReadyMsg=セットアップは、お使いのコンピューター上の %1 をバージョン %2 に更新する準備が整いました。
japanese.UpdateFinishedHeading=%1 の更新が完了しました
japanese.UpdateFinishedMsg=%1 は正常に更新されました。%n%n[完了] をクリックしてセットアップを終了してください。

korean.StartupTask=Windows 시작 시 Uzume 자동 실행
korean.StartupGroup=시작 옵션:
korean.UpdateAppTitle=%1 업데이트
korean.ButtonUpdate=업데이트
korean.UpdateReadyMsg=컴퓨터의 %1을(를) %2 버전으로 업데이트할 준비가 되었습니다.
korean.UpdateFinishedHeading=%1 업데이트 완료
korean.UpdateFinishedMsg=%1이(가) 성공적으로 업데이트되었습니다.%n%n[종료]를 클릭하여 설치 프로그램을 마칩니다.

chinesesimplified.StartupTask=随 Windows 自动启动 Uzume
chinesesimplified.StartupGroup=启动选项:
chinesesimplified.UpdateAppTitle=更新 %1
chinesesimplified.ButtonUpdate=更新
chinesesimplified.UpdateReadyMsg=安装程序已准备好将计算机上的 %1 更新至版本 %2。
chinesesimplified.UpdateFinishedHeading=%1 更新完成
chinesesimplified.UpdateFinishedMsg=%1 已成功更新至您的计算机。%n%n单击“完成”退出安装程序。

chinesetraditional.StartupTask=隨 Windows 自動啟動 Uzume
chinesetraditional.StartupGroup=啟動選項:
chinesetraditional.UpdateAppTitle=更新 %1
chinesetraditional.ButtonUpdate=更新
chinesetraditional.UpdateReadyMsg=安裝程式已準備好將電腦上的 %1 更新至版本 %2。
chinesetraditional.UpdateFinishedHeading=%1 更新完成
chinesetraditional.UpdateFinishedMsg=%1 已成功更新至您的電腦。%n%n按一下「完成」結束安裝程式。

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startupicon"; Description: "{cm:StartupTask}"; GroupDescription: "{cm:StartupGroup}"; Flags: unchecked

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

[Code]
var
  IsUpdateInstall: Boolean;

function IsAppInstalled: Boolean;
var
  UninstallKey: String;
begin
  UninstallKey := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#SetupSetting("AppId")}_is1';
  Result := RegKeyExists(HKCU, UninstallKey) or
            RegKeyExists(HKLM, UninstallKey) or
            RegKeyExists(HKLM64, UninstallKey);
end;

procedure InitializeWizard;
begin
  IsUpdateInstall := IsAppInstalled;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  if IsUpdateInstall then
  begin
    WizardForm.Caption := FmtMessage(CustomMessage('UpdateAppTitle'), ['{#MyAppName}']);

    if CurPageID = wpReady then
    begin
      WizardForm.NextButton.Caption := CustomMessage('ButtonUpdate');
      WizardForm.ReadyLabel.Caption := FmtMessage(CustomMessage('UpdateReadyMsg'), ['{#MyAppName}', '{#MyAppVersion}']);
    end
    else if CurPageID = wpFinished then
    begin
      WizardForm.FinishedHeadingLabel.Caption := FmtMessage(CustomMessage('UpdateFinishedHeading'), ['{#MyAppName}']);
      WizardForm.FinishedLabel.Caption := FmtMessage(CustomMessage('UpdateFinishedMsg'), ['{#MyAppName}']);
    end;
  end;
end;
