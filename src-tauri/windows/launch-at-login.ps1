$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$startupIdentity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
$startupSid = $startupIdentity.User.Value
$startupTaskName = 'TunnelDesk-' + $startupSid
$startupExe = $env:TUNNELDESK_STARTUP_EXE
$startupTask = Get-ScheduledTask -TaskName $startupTaskName -TaskPath '\' -ErrorAction SilentlyContinue
$startupDescription = 'TunnelDesk login startup (com.tunneldesk.app)'

if ($startupTask -and $startupTask.Description -ne $startupDescription) {
    throw 'A different application owns the TunnelDesk startup task name.'
}

switch ($env:TUNNELDESK_STARTUP_OPERATION) {
    'query' {
        $startupEnabled = $startupTask -and $startupTask.State -ne 'Disabled' `
            -and $startupTask.Actions.Count -eq 1 `
            -and $startupTask.Actions[0].Execute -eq $startupExe `
            -and -not $startupTask.Actions[0].Arguments `
            -and $startupTask.Principal.UserId -in @($startupSid, $startupIdentity.Name) `
            -and $startupTask.Principal.LogonType -eq 'Interactive' `
            -and $startupTask.Principal.RunLevel -eq 'Highest'
        [bool]$startupEnabled | ConvertTo-Json -Compress
    }
    'enable' {
        $startupAction = New-ScheduledTaskAction -Execute $startupExe
        $startupTrigger = New-ScheduledTaskTrigger -AtLogOn -User $startupSid
        $startupPrincipal = New-ScheduledTaskPrincipal -UserId $startupSid -LogonType Interactive -RunLevel Highest
        $startupSettings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
            -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
        Register-ScheduledTask -TaskName $startupTaskName -TaskPath '\' -Action $startupAction `
            -Trigger $startupTrigger -Principal $startupPrincipal -Settings $startupSettings `
            -Description $startupDescription -Force | Out-Null
    }
    'disable' {
        if ($startupTask) {
            Unregister-ScheduledTask -TaskName $startupTaskName -TaskPath '\' -Confirm:$false
        }
    }
    default { throw 'Unsupported TunnelDesk startup operation.' }
}
