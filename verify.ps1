$ErrorActionPreference='Stop'
$binary=Join-Path $PSScriptRoot 'target\release\tcl-brightness.exe'
$work=Join-Path $PSScriptRoot 'work'
New-Item -ItemType Directory -Force -Path $work | Out-Null
function Invoke-Brightness([string[]]$Arguments,[string]$Name){
    $result=Join-Path $work $Name
    $process=Start-Process -FilePath $binary -ArgumentList ($Arguments+@('--out',('"'+$result+'"'))) -Wait -PassThru -WindowStyle Hidden
    if($process.ExitCode -ne 0){throw "Brightness command failed: $(Get-Content -LiteralPath $result -Raw)"}
    Get-Content -LiteralPath $result -Raw
}
Invoke-Brightness @('--self-test') 'selftest.txt'
$before=Invoke-Brightness @('--status') 'before.json' | ConvertFrom-Json
try{
    $requested=if($before.brightness -gt 0){$before.brightness-1}else{1}
    $after=Invoke-Brightness @('--set',"$requested") 'changed.json' | ConvertFrom-Json
    if($after.brightness -ne $requested -or $after.mode -ne $before.mode){throw 'Brightness read-back failed'}
    'Monitor brightness changed and read back successfully.'
}finally{
    $restored=Invoke-Brightness @('--set',"$($before.brightness)") 'restored.json' | ConvertFrom-Json
    if($restored.brightness -ne $before.brightness){throw 'Original brightness was not restored'}
    'Original brightness restored.'
}
