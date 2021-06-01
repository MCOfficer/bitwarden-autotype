#define AppName "bitwarden-autotype"  
#define AppExeName "bitwarden-autotype.exe"
#define AppExeSource ".\target\release\" + AppExeName
#define AppVersion RemoveFileExt(GetVersionNumbersString(AppExeSource))
#define AppPublisher "MCOfficer"
#define AppURL "https://github.com/MCOfficer/bitwarden-autotype/"

[Setup]
AppId={{2B0E41DE-5F03-4B1E-87CD-56BA81C740B9}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={userpf}\{#AppName}
DefaultGroupName={#AppName}
LicenseFile=.\LICENSE
OutputDir=.
OutputBaseFilename={#AppName}-{#AppVersion}-setup
Compression=lzma
SolidCompression=yes
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#AppExeSource}"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
Name: autostart; Description: "Launch automatically when Windows starts"

[Icons]
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}" 
Name: "{userstartup}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: autostart

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
