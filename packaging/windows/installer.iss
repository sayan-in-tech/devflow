[Setup]
AppName=devflow
AppVersion=0.1.0
DefaultDirName={pf}\devflow
OutputDir=.
OutputBaseFilename=devflow-setup

[Files]
Source: "..\..\target\release\devflow.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\devflow"; Filename: "{app}\devflow.exe"
