$ErrorActionPreference = 'Stop'

function Assert-That([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "assertion failed: $Message" }
}

# This is a disposable model, not a proposed API.  It makes the information
# that an API would need to preserve observable: immediate runnable work, a
# host-domain deadline, and external completions can coexist.
function New-Session([int]$Generation) {
    [pscustomobject]@{
        Generation = $Generation
        Cancelled = $false
        Immediate = $false
        DeadlineMs = $null
        External = [System.Collections.Generic.HashSet[string]]::new()
        SeenCompletions = [System.Collections.Generic.HashSet[string]]::new()
        Revision = 0
    }
}

function Demand-Of($Session) {
    [pscustomobject]@{
        Immediate = $Session.Immediate
        DeadlineMs = $Session.DeadlineMs
        External = @($Session.External | Sort-Object)
        Revision = $Session.Revision
    }
}

function Complete-External($Session, [int]$Generation, [string]$Token) {
    if ($Session.Cancelled -or $Session.Generation -ne $Generation) { return 'stale-refused' }
    if (-not $Session.External.Remove($Token)) { return 'duplicate-or-unknown-refused' }
    if (-not $Session.SeenCompletions.Add($Token)) { return 'duplicate-or-unknown-refused' }
    $Session.Immediate = $true
    $Session.Revision++
    return 'accepted'
}

function Query-DriveDemand($Session) {
    # A host obtains this before arming its platform wake source.
    return Demand-Of $Session
}

function Register-Wake($Session) {
    # Registration is deliberately separate from querying. The token is the
    # last observed session generation/revision, not a callback implementation.
    return [pscustomobject]@{
        Generation = $Session.Generation
        Revision = $Session.Revision
    }
}

function Recheck-AfterRegistration($Session, $Registration) {
    # A changed generation means the registration cannot drive this session.
    # A changed revision or runnable work means completion raced registration;
    # the host must queue an immediate drive rather than sleep.
    if ($Session.Generation -ne $Registration.Generation -or $Session.Cancelled) {
        return 'stale-registration'
    }
    if ($Session.Revision -ne $Registration.Revision -or $Session.Immediate) {
        return 'drive-now'
    }
    return 'armed-and-idle'
}

$session = New-Session 7
$session.Immediate = $true
$session.DeadlineMs = 25
[void]$session.External.Add('worker:4')
[void]$session.External.Add('fetch:9')
$session.Revision++
$demand = Demand-Of $session
Assert-That $demand.Immediate 'immediate work is reported'
Assert-That ($demand.DeadlineMs -eq 25) 'deadline is reported without choosing a clock'
Assert-That ($demand.External.Count -eq 2) 'two external sources coexist with immediate/deadline work'

# Negative control: settled polling observes only a yes/no state.  Both of
# these sessions are unsettled, but require different host behavior.
$deadlineOnly = New-Session 8
$deadlineOnly.DeadlineMs = 100
$externalOnly = New-Session 9
[void]$externalOnly.External.Add('worker:2')
$pollDeadline = -not (($deadlineOnly.DeadlineMs -eq $null) -and ($deadlineOnly.External.Count -eq 0) -and (-not $deadlineOnly.Immediate))
$pollExternal = -not (($externalOnly.DeadlineMs -eq $null) -and ($externalOnly.External.Count -eq 0) -and (-not $externalOnly.Immediate))
Assert-That ($pollDeadline -and $pollExternal) 'polling observes both as merely unsettled'
Assert-That ((Demand-Of $deadlineOnly).DeadlineMs -eq 100) 'deadline-only does not invent external work'
Assert-That ((Demand-Of $externalOnly).External.Count -eq 1) 'external-only does not invent a deadline'

# Registration/completion race: query says idle, then completion occurs before
# platform registration is effective. A callback-only scheme loses that wake:
# no listener saw completion and no later event occurs. Query/register/recheck
# observes the changed revision and drives immediately.
$race = New-Session 10
[void]$race.External.Add('worker:race')
$queried = Query-DriveDemand $race
Assert-That (-not $queried.Immediate) 'race starts with no runnable work'
# Negative control: completion before callback registration is delivered to no
# listener; a scheme that does not reread now sleeps despite runnable work.
$callbackRegistered = $false
Assert-That ((Complete-External $race 10 'worker:race') -eq 'accepted') 'completion before wake registration is accepted'
$missedCallback = -not $callbackRegistered
$callbackRegistered = $true
$noRecheckSleeps = $missedCallback
Assert-That ($noRecheckSleeps -and $race.Immediate) 'negative control loses a wake without recheck'

$race2 = New-Session 10
[void]$race2.External.Add('worker:race')
$registration = Register-Wake $race2
Assert-That ((Complete-External $race2 10 'worker:race') -eq 'accepted') 'completion racing registered wake is accepted'
Assert-That ((Recheck-AfterRegistration $race2 $registration) -eq 'drive-now') 'query/register/recheck closes lost-wake race'

# Navigation is cancellation plus generation replacement. Old completion must
# not mutate the new session; live completion is delivered once.
$old = New-Session 11
[void]$old.External.Add('worker:old')
$old.Cancelled = $true
$new = New-Session 12
[void]$new.External.Add('worker:new')
Assert-That ((Complete-External $old 11 'worker:old') -eq 'stale-refused') 'navigation refuses old-session completion'
Assert-That ((Complete-External $new 12 'worker:new') -eq 'accepted') 'live-session completion is accepted'
Assert-That ((Complete-External $new 12 'worker:new') -eq 'duplicate-or-unknown-refused') 'duplicate live completion is refused'

'R1 drive/wake model: PASS'
