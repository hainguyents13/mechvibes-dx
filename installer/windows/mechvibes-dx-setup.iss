; MechvibesDX Inno Setup Script
; Requires Inno Setup 6.0 or later: https://jrsoftware.org/isinfo.php
;
; Version is NOT hardcoded here - pass it at compile time:
;   ISCC.exe /DAppVersion=0.5.2 installer\windows\mechvibes-dx-setup.iss
; scripts/build-windows-installer.ps1 does this automatically by reading
; Cargo.toml, so there is a single source of truth for the version.

#ifndef AppVersion
  #error AppVersion must be defined, e.g. ISCC /DAppVersion=0.5.2 mechvibes-dx-setup.iss
#endif

#define MyAppName "MechvibesDX"
#define MyAppVersion AppVersion
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
LicenseFile=..\..\LICENSE
; OutputBaseFilename MUST contain "x64" and end in ".exe" to match the
; in-app auto-updater's asset filter (src/utils/auto_updater.rs).
OutputDir=..\..\dist
OutputBaseFilename=MechvibesDX-{#MyAppVersion}-Setup-x64
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
CloseApplications=yes
CloseApplicationsFilter=*.exe
RestartApplications=yes

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

; No [Files] entry for data/ - it's not part of the source tree (config.json,
; manifest.json, etc. are runtime state, not shipped defaults). The app
; creates data/ itself on first run and falls back to defaults for any
; missing file (see AppConfig::load() in src/state/config.rs), so nothing
; needs to be bundled here.

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; IconIndex: 0
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; IconIndex: 0; Tasks: desktopicon

[Registry]
; Add to startup if task is selected
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"" --minimized"; Flags: uninsdeletevalue; Tasks: startup

[Run]
; Interactive installs: the usual "run it now" checkbox on the last page.
; `skipifsilent` means this does NOT fire during an auto-update, which runs
; with /VERYSILENT and has no final page - the relaunch there comes from
; CloseApplications + RestartApplications above (Restart Manager records the
; running app before closing it and starts it again afterwards), which the
; in-app updater triggers by passing /RESTARTAPPLICATIONS.
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

; Silent installs (i.e. auto-update) only: this is what actually relaunches
; the app, and it is NOT redundant with RestartApplications above.
;
; RestartApplications only restarts processes that called the Windows
; RegisterApplicationRestart API - verified in the Inno 6 docs for the
; directive. MechvibesDX never calls it, so Restart Manager closes the app
; but will not bring it back. Without this entry an auto-update would leave
; the user with no running app and no window explaining why.
;
; `runasoriginaluser` matters because Setup may be elevated: the app has to
; come back as the user who owns the tray/audio session, not as admin.
; `RmSessionStarted` is guarded against so we never end up with two copies
; running in the case where Restart Manager did handle the relaunch.
Filename: "{app}\{#MyAppExeName}"; Flags: nowait runasoriginaluser skipifnotsilent; Check: not RmSessionStarted

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\{#MyAppName}"
Type: filesandordirs; Name: "{userappdata}\Mechvibes"

[Code]
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
