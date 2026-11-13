$ErrorActionPreference='Stop'
$source=Join-Path $PSScriptRoot 'target\release\tcl-brightness.exe'
if(!(Test-Path -LiteralPath $source)){throw 'Run build.ps1 first'}
$folder=Join-Path $PSScriptRoot 'bin'
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$binary=Join-Path $folder 'TCL Brightness.exe'
$process=Start-Process -FilePath $source -ArgumentList '--quit' -PassThru -WindowStyle Hidden
$process.WaitForExit()
$running=Get-Process -Name 'TCL Brightness','tcl-brightness' -ErrorAction SilentlyContinue
foreach($item in $running){if($item.Path -eq $binary -or $item.Path -eq $source){$item.WaitForExit(3000)|Out-Null}}
Copy-Item -LiteralPath $source -Destination $binary -Force
$shell=New-Object -ComObject WScript.Shell
foreach($linkPath in @((Join-Path ([Environment]::GetFolderPath('Startup')) 'TCL Brightness.lnk'),(Join-Path ([Environment]::GetFolderPath('Programs')) 'TCL Brightness.lnk'))){
    $link=$shell.CreateShortcut($linkPath)
    $link.TargetPath=$binary
    $link.WorkingDirectory=$folder
    $link.IconLocation="$binary,0"
    $link.Save()
}
Start-Process -FilePath $binary -ArgumentList '--show'
'Installed TCL Brightness, added Start menu and sign-in shortcuts, and opened the slider.'
