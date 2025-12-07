; MechvibesDX Inno Setup Script
; Requires Inno Setup 6.0 or later: https://jrsoftware.org/isinfo.php

#define MyAppName "MechvibesDX"
#define MyAppVersion "0.4.0"
#define MyAppPublisher "Hai Nguyen"
#define MyAppURL "https://github.com/hainguyents13/mechvibes-dx"
#define MyAppExeName "mechvibes-dx.exe"
#define MyAppId "{{8B5F5E5E-5E5E-5E5E-5E5E-5E5E5E5E5E5E}"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=MechvibesDX-{#MyAppVersion}-Setup
SetupIconFile=..\..\assets\icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}
DisableProgramGroupPage=yes
DisableWelcomePage=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Run at Windows startup"; GroupDescription: "Additional options:"; Flags: unchecked

[Files]
; Main executable
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; Assets folder - MUST be in the same directory as the executable for Dioxus asset!() macro
Source: "..\..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

; Soundpacks folder (built-in soundpacks)
Source: "..\..\soundpacks\*"; DestDir: "{app}\soundpacks"; Flags: ignoreversion recursesubdirs createallsubdirs

; Data folder (default config files) - only if doesn't exist to preserve user data
Source: "..\..\data\*"; DestDir: "{app}\data"; Flags: ignoreversion onlyifdoesntexist recursesubdirs createallsubdirs

; Documentation
;Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
; Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; IconIndex: 0
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; IconIndex: 0; Tasks: desktopicon

[Registry]
; Add to startup if task is selected
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"" --minimized"; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\{#MyAppName}"
Type: filesandordirs; Name: "{userappdata}\Mechvibes"

[Code]
function InitializeSetup(): Boolean;
begin
  Result := True;
end;

procedure RefreshIconCache();
var
  ResultCode: Integer;
begin
  Exec('ie4uinit.exe', '-show', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    // Create custom soundpacks directory in AppData
    CreateDir(ExpandConstant('{userappdata}\Mechvibes\soundpacks\keyboard'));
    CreateDir(ExpandConstant('{userappdata}\Mechvibes\soundpacks\mouse'));

    // Force Windows to refresh icon cache
    RefreshIconCache();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ResultCode: Integer;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if MsgBox('Do you want to remove all user data including custom soundpacks and settings?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      DelTree(ExpandConstant('{userappdata}\Mechvibes'), True, True, True);
    end;
  end;
end;