; Per-user fork installer. The existing portable layout isolates its profile
; from stock Spotifast and keeps the same profile on every update/reinstall.
#ifndef Version
  #error Version is required
#endif
#ifndef Binary
  #error Binary is required
#endif
#ifndef OutputDir
  #error OutputDir is required
#endif
; Tests override only these identities, compiling this same production script.
#ifndef ProductId
  #define ProductId "SpotifastLucid-08DFAE02-7C24-481A-AABD-71D675B4F0B4"
#endif
#ifndef ProductName
  #define ProductName "Spotifast Lucid"
#endif

[Setup]
AppId={#ProductId}
AppName={#ProductName}
AppVersion={#Version}
AppVerName={#ProductName} {#Version}
AppPublisher=SxGoldenxS
AppPublisherURL=https://github.com/SxGoldenxS/Spotifast-Lucid
AppSupportURL=https://github.com/SxGoldenxS/Spotifast-Lucid/issues
AppUpdatesURL=https://github.com/SxGoldenxS/Spotifast-Lucid/releases/latest
DefaultDirName={localappdata}\Programs\{#ProductName}
DefaultGroupName={#ProductName}
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
DisableProgramGroupPage=yes
DisableReadyPage=yes
WizardStyle=modern
SetupIconFile=slime.ico
UninstallDisplayIcon={app}\spotifast.exe
CloseApplications=no
RestartApplications=no
Compression=lzma2/max
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=Spotifast-Lucid-Windows-x64-Setup
VersionInfoVersion={#Version}.0

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#Binary}"; DestDir: "{app}"; DestName: "spotifast.exe"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\FORK.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\INSTALL-LUCID.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup-help.txt"; DestDir: "{app}"; DestName: "Install guide.txt"; Flags: ignoreversion
; Never bundle a profile, delete one, or replace existing user settings/palettes.
Source: "..\..\packaging\spotifast-portable.txt"; DestDir: "{app}"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "default-settings.json"; DestDir: "{app}\spotifast-data\config"; DestName: "settings.json"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "TrueVision.json"; DestDir: "{app}\spotifast-data\config\themes"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "GreatSage.json"; DestDir: "{app}\spotifast-data\config\themes"; Flags: onlyifdoesntexist uninsneveruninstall

[Icons]
Name: "{userprograms}\{#ProductName}"; Filename: "{app}\spotifast.exe"; WorkingDir: "{app}"
Name: "{userdesktop}\{#ProductName}"; Filename: "{app}\spotifast.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\spotifast.exe"; Description: "Launch {#ProductName}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[Code]
function ProgramAvailable(const Filename: String): Boolean;
var
  Probe: TFileStream;
begin
  Result := True;
  if not FileExists(Filename) then Exit;
  try
    Probe := TFileStream.Create(Filename, fmOpenReadWrite or fmShareExclusive);
    Probe.Free;
  except
    Result := False;
  end;
end;

function ProgramFilesAvailable: Boolean;
begin
  Result := ProgramAvailable(ExpandConstant('{app}\spotifast.exe')) and
    ProgramAvailable(ExpandConstant('{app}\spotifast-lucid.exe'));
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  if FileExists(ExpandConstant('{app}\spotifast-installer.txt')) then
    Result := 'This folder belongs to the original Spotifast installer. Choose a separate folder for Spotifast Lucid.'
  else if not ProgramFilesAvailable then
    Result := 'The program file is in use or inaccessible. Quit Spotifast completely with Ctrl+Q, then click Retry. No program files or settings have been changed.';
end;

function InitializeUninstall: Boolean;
begin
  Result := ProgramFilesAvailable;
  if not Result and not UninstallSilent then
    MsgBox('Quit Spotifast completely with Ctrl+Q, then run uninstall again. Your profile will be kept.', mbInformation, MB_OK);
end;
