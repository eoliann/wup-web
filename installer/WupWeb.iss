#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef AppBinary
  #error AppBinary is required
#endif
#ifndef OutputPath
  #define OutputPath "Output"
#endif

[Setup]
AppId=Wup Web
AppName=Wup Web
AppVersion={#AppVersion}
AppPublisher=eoliann
DefaultDirName={autopf}\Wup Web
DefaultGroupName=Wup Web
ArchitecturesAllowed=x64compatible
OutputDir={#OutputPath}
OutputBaseFilename=Wup_Web_Installer
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\wup-web.exe
UninstallDisplayName=Wup Web
CloseApplications=force
CloseApplicationsFilter=wup-web.exe
AppMutex={code:UninstallAppMutex}
Compression=lzma2
SolidCompression=yes

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#AppBinary}"; DestDir: "{app}"; DestName: "wup-web.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\Wup Web"; Filename: "{app}\wup-web.exe"
Name: "{autodesktop}\Wup Web"; Filename: "{app}\wup-web.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\wup-web.exe"; Description: "Launch Wup Web"; Flags: nowait postinstall skipifsilent

[Code]
function UninstallAppMutex(Param: String): String;
begin
  { Let Restart Manager close the installed executable during an upgrade. }
  Result := '';
  if IsUninstaller then
    Result := 'Local\eoliann.wup-web';
end;
