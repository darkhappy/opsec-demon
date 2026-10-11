# Read Apple Music's Windows media session; stream one UTF-8 JSON object per second.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$managerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$propertiesType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties, Windows.Media.Control, ContentType = WindowsRuntime]
$asTask = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
    $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetParameters().Count -eq 1 -and
    $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
} | Select-Object -First 1
function Await-WinRT($operation, $resultType) {
    $task = $asTask.MakeGenericMethod($resultType).Invoke($null, @($operation))
    if (-not $task.Wait(5000)) { throw 'Windows media API timed out' }
    return $task.Result
}
$manager = Await-WinRT ($managerType::RequestAsync()) $managerType
while ($true) {
    try {
        $allSessions = @($manager.GetSessions())
        $sessions = @($allSessions | Where-Object {
            $_.SourceAppUserModelId -match '(?i)AppleMusic|AppleInc\.AppleMusic'
        })
        $session = $sessions | Where-Object {
            $_.GetPlaybackInfo().PlaybackStatus.ToString() -eq 'Playing'
        } | Select-Object -First 1
        if ($null -eq $session) { $session = $sessions | Select-Object -First 1 }
        $state = @{ status = 'Stopped'; title = ''; artist = ''; album = ''; source = '' }
        if ($null -ne $session) {
            $properties = Await-WinRT ($session.TryGetMediaPropertiesAsync()) $propertiesType
            $state = @{
                status = $session.GetPlaybackInfo().PlaybackStatus.ToString()
                title = $properties.Title
                artist = $properties.Artist
                album = $properties.AlbumTitle
                source = $session.SourceAppUserModelId
            }
        }
        $state.sessions = @($allSessions | ForEach-Object { $_.SourceAppUserModelId })
        [Console]::WriteLine(($state | ConvertTo-Json -Compress))
        [Console]::Out.Flush()
    } catch {
        [Console]::Error.WriteLine($_.Exception.Message)
        [Console]::WriteLine('{"status":"Stopped","title":"","artist":"","album":"","source":""}')
        [Console]::Out.Flush()
    }
    Start-Sleep -Seconds 1
}
