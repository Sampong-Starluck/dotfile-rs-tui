### PowerShell template profile
### Version 1.00 - Lim Sampong <sampong.lim@gmail.com>

# Find out if the current user identity is elevated (has admin rights)
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal $identity
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# Custom override function
#$override_file = "$env:APPDATA\Sampong_dotfile\PowerShell\override_function.ps1"

# If so and the current host is a command line, then change to red color
# as warning to user that they are operating in an elevated context
# Useful shortcuts for traversing directories
function cd... { Set-Location ..\.. }
function cd.... { Set-Location ..\..\.. }

# Compute file hashes - useful for checking successful downloads
function md5 { Get-FileHash -Algorithm MD5 $args }
function sha1 { Get-FileHash -Algorithm SHA1 $args }
function sha256 { Get-FileHash -Algorithm SHA256 $args }

# Quick shortcut to start notepad
function n { notepad $args }

# Set up command prompt and window title. Use UNIX-style convention for identifying
# whether user is elevated (root) or not. Window title shows current version of PowerShell
# and appends [ADMIN] if appropriate for easy taskbar identification
function prompt {
    if ($isAdmin) {
        "[" + (Get-Location) + "] # "
    } else {
        "[" + (Get-Location) + "] $ "
    }
}

# Check dir existing
function Test_DirectoryExists {
    param(
        [string]$Path, [switch]$Verbose
    )
    $exists = Test-Path -Path $Path -PathType Container

    if ($exists)
    {
        if ($Verbose)
        {
            Write-Host "Directory exists: $Path" -ForegroundColor Green
        }
        return $true
    }
    else
    {
        if ($Verbose) {
            Write-Host "Directory does not exist: $Path" -ForegroundColor Red
        }
        return $false
    }
}

function Test-FileExists {
    param(
        [string]$Path,
        [switch]$Verbose
    )

    $exists = Test-Path -Path $Path -PathType Leaf

    if ($exists) {
        if ($Verbose) {
            Write-Host "File exists: $Path" -ForegroundColor Green
        }
        return $true
    } else {
        if ($Verbose) {
            Write-Host "File does not exist: $Path" -ForegroundColor Red
        }
        return $false
    }
}

$Host.UI.RawUI.WindowTitle = "PowerShell {0}" -f $PSVersionTable.PSVersion.ToString()
if ($isAdmin) {
    $Host.UI.RawUI.WindowTitle += " [ADMIN]"
}

# Does the the rough equivalent of dir /s /b. For example, dirs *.png is dir /s /b *.png
function dirs {
    if ($args.Count -gt 0) {
        Get-ChildItem -Recurse -Include "$args" | Foreach-Object FullName
    } else {
        Get-ChildItem -Recurse | Foreach-Object FullName
    }
}

# Simple function to start a new elevated process. If arguments are supplied then
# a single command is started with admin rights; if not then a new admin instance
# of PowerShell is started.
function admin {
    if ($args.Count -gt 0) {
        $argList = "& '" + $args + "'"
        Start-Process "$psHome\pwsh.exe" -Verb runAs -ArgumentList $argList
    } else {
        Start-Process "$psHome\pwsh.exe" -Verb runAs
    }
}

# Set UNIX-like aliases for the admin command, so sudo <command> will run the command
# with elevated rights.
Set-Alias -Name su -Value admin
# Set-Alias -Name sudo -Value admin

<#
   Test-CommandExists function is used for find and test the command line tool the installed in Windows
 #>
Function Test-CommandExists {
    Param ($command)
    $oldPreference = $ErrorActionPreference
    $ErrorActionPreference = 'SilentlyContinue'
    try { if (Get-Command $command) { RETURN $true } }
    Catch { Write-Host "$command does not exist"; RETURN $false }
    Finally { $ErrorActionPreference = $oldPreference }
}
#
# Aliases
#
# If your favorite editor is not here, add an elseif and ensure that the directory it is installed in exists in your $env:Path
#
# Editor Configuration
if ($EDITOR_Override){
    $EDITOR = $EDITOR_Override
} else {
    $EDITOR = if (Test-CommandExists nvim) { 'nvim' }
    elseif (Test-CommandExists pvim) { 'pvim' }
    elseif (Test-CommandExists vim) { 'vim' }
    elseif (Test-CommandExists vi) { 'vi' }
    elseif (Test-CommandExists code) { 'code' }
    elseif (Test-CommandExists codium) { 'codium' }
    elseif (Test-CommandExists notepad++) { 'notepad++' }
    elseif (Test-CommandExists sublime_text) { 'sublime_text' }
    elseif (Test-CommandExists sublime_text) { 'edit' }
    else { 'notepad' }
    Set-Alias -Name vim -Value $EDITOR
}

# We don't need these any more; they were just temporary variables to get to $isAdmin.
# Delete them to prevent cluttering up the user profile.
Remove-Variable identity
Remove-Variable principal

function ll { Get-ChildItem -Path $pwd -File }
function g { Set-Location $HOME\Documents\Github }
function gpull{
    param ($argument)
    if ([string]::IsNullOrEmpty($argument)) {
        git pull
    } else {
        git pull origin $argument
    }

}
function glog {
    param ($command)
    if ($command -eq "deco") {
        git log --graph --oneline --decorate
    } else {
        git log
    }

}
function gcom {
        git add .
        git commit -m "$args"
}
function gamend {
        git add .
        git commit --amend --no-edit
}
# function lazygcom {
#         git add .
#         git commit -m "$args"
#         git pull
#         git push
# }
function lazygcom {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $false)]
        [ValidateNotNullOrEmpty()]
        [Alias("b")]
        [string]$Branch,

        [Parameter(Mandatory = $false)]
        [ValidateNotNullOrEmpty()]
        [Alias("m")]
        [string]$Message = "daily update"
    )

    try {
        # Stage all changes
        git add .

        # Commit with message
        git commit -m $Message

        # Pull from origin
        git pull

        # If branch specified, pull from that specific branch
        if ($Branch) {
            git pull origin $Branch
        }

        # Push to origin
        git push
    }
    catch {
        Write-Error "Git operation failed: $_"
    }
}

function gcbranch {
    param(
        [string]$branch,
        [switch][Alias("b")]$bugs,
        [switch][Alias("f")]$feats,
        [switch][Alias("h")]$hotfix
    )

    if ($null -ne $branch -and $branch -ne "") {
        if ($hotfix -or $feats -or $bugs) {
            # If any flag is passed, create branch with prefix
            $branch_type = if ($hotfix) {
                "hotfix"
            } elseif ($feats) {
                "feats"
            } elseif ($bugs) {
                "bugs"
            }

            git branch "$branch_type/$branch"
            git checkout "$branch_type/$branch"
        } else {
            # No flag passed, just checkout the branch directly
            git checkout $branch
        }
    } else {
        git branch
    }
}

function lazygamend {
        git add .
        git commit --amend --no-edit
        git pull
        git push
}
function Get-PubIP {
    (Invoke-WebRequest http://ifconfig.me/ip ).Content
}

# Start Ngork with multiple ports: (cmd: Start-Ngrok 8000 8080 8081)
function Start-Ngrok {
    param (
        [Parameter(Mandatory=$true, ValueFromRemainingArguments=$true)]
        [int[]]$Ports
    )

    if (-not $Ports) {
        throw "You must provide at least one port number."
    }

    foreach ($Port in $Ports) {
        ngrok http $Port --request-header-add ngrok-skip-browser-warning:true
    }
}

# Open WinUtil full-release
function winutil {
    irm https://christitus.com/win | iex
}

# command surreal use for start surrealdb in docker namae surreal_database in port 8000
# function surreal {
#     param ($command)
#     if ($command -eq "start") {
#         docker run --rm --pull always --name surreal_database -p 8000:8000 surrealdb/surrealdb:latest $command
#     } else {
#         Write-Output "This command is invalid, Exit surrealDB use ctl+c shortcut."
#     }
#
# }

function uptime {
    try {
        # find date/time format
        $dateFormat = [System.Globalization.CultureInfo]::CurrentCulture.DateTimeFormat.ShortDatePattern
        $timeFormat = [System.Globalization.CultureInfo]::CurrentCulture.DateTimeFormat.LongTimePattern

        # check powershell version
        if ($PSVersionTable.PSVersion.Major -eq 5) {
            $lastBoot = (Get-WmiObject win32_operatingsystem).LastBootUpTime
            $bootTime = [System.Management.ManagementDateTimeConverter]::ToDateTime($lastBoot)

            # reformat lastBoot
            $lastBoot = $bootTime.ToString("$dateFormat $timeFormat")
        } else {
            # the Get-Uptime cmdlet was introduced in PowerShell 6.0
            $lastBoot = (Get-Uptime -Since).ToString("$dateFormat $timeFormat")
            $bootTime = [System.DateTime]::ParseExact($lastBoot, "$dateFormat $timeFormat", [System.Globalization.CultureInfo]::InvariantCulture)
        }

        # Format the start time
        $formattedBootTime = $bootTime.ToString("dddd, MMMM dd, yyyy HH:mm:ss", [System.Globalization.CultureInfo]::InvariantCulture) + " [$lastBoot]"
        Write-Host "System started on: $formattedBootTime" -ForegroundColor DarkGray

        # calculate uptime
        $uptime = (Get-Date) - $bootTime

        # Uptime in days, hours, minutes, and seconds
        $days = $uptime.Days
        $hours = $uptime.Hours
        $minutes = $uptime.Minutes
        $seconds = $uptime.Seconds

        # Uptime output
        Write-Host ("Uptime: {0} days, {1} hours, {2} minutes, {3} seconds" -f $days, $hours, $minutes, $seconds) -ForegroundColor Blue

    } catch {
        Write-Error "An error occurred while retrieving system uptime."
    }
}

function restart-profile {
    & $profile
}
function find-file($name) {
    Get-ChildItem -recurse -filter "*${name}*" -ErrorAction SilentlyContinue | ForEach-Object {
        $place_path = $_.directory
        Write-Output "${place_path}\${_}"
    }
}
function unzip ($file) {
    Write-Output("Extracting", $file, "to", $pwd)
    $fullFile = Get-ChildItem -Path $pwd -Filter .\cove.zip | ForEach-Object { $_.FullName }
    Expand-Archive -Path $fullFile -DestinationPath $pwd
}
function grep($regex, $dir) {
    if ( $dir ) {
        Get-ChildItem $dir | select-string $regex
        return
    }
    $input | select-string $regex
}
function touch($file) {
    "" | Out-File $file -Encoding ASCII
}
function df {
    get-volume
}
function sed($file, $find, $replace) {
    (Get-Content $file).replace("$find", $replace) | Set-Content $file
}
function which($name) {
    Get-Command $name | Select-Object -ExpandProperty Definition
}
function export($name, $value) {
    set-item -force -path "env:$name" -value $value;
}
function pkill($name) {
    Get-Process $name -ErrorAction SilentlyContinue | Stop-Process
}
function pgrep($name) {
    Get-Process $name
}
function pport {
    netstat -aon | findstr "$args"
}

function Clear-Cache {
    # add clear cache logic here
    Write-Host "Clearing cache..." -ForegroundColor Cyan

    # Clear Windows Prefetch
    Write-Host "Clearing Windows Prefetch..." -ForegroundColor Yellow
    Remove-Item -Path "$env:SystemRoot\Prefetch\*" -Force -ErrorAction SilentlyContinue

    # Clear Windows Temp
    Write-Host "Clearing Windows Temp..." -ForegroundColor Yellow
    Remove-Item -Path "$env:SystemRoot\Temp\*" -Recurse -Force -ErrorAction SilentlyContinue

    # Clear User Temp
    Write-Host "Clearing User Temp..." -ForegroundColor Yellow
    Remove-Item -Path "$env:TEMP\*" -Recurse -Force -ErrorAction SilentlyContinue

    # Clear Internet Explorer Cache
    Write-Host "Clearing Internet Explorer Cache..." -ForegroundColor Yellow
    Remove-Item -Path "$env:LOCALAPPDATA\Microsoft\Windows\INetCache\*" -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host "Cache clearing completed." -ForegroundColor Green
}

# Enhanced PowerShell Experience
# Enhanced PSReadLine Configuration
$PSReadLineOptions = @{
    EditMode = 'Windows'
    HistoryNoDuplicates = $true
    HistorySearchCursorMovesToEnd = $true
    Colors = @{
        Command = '#87CEEB'  # SkyBlue (pastel)
        Parameter = '#98FB98'  # PaleGreen (pastel)
        Operator = '#FFB6C1'  # LightPink (pastel)
        Variable = '#DDA0DD'  # Plum (pastel)
        String = '#FFDAB9'  # PeachPuff (pastel)
        Number = '#B0E0E6'  # PowderBlue (pastel)
        Type = '#F0E68C'  # Khaki (pastel)
        Comment = '#D3D3D3'  # LightGray (pastel)
        Keyword = '#8367c7'  # Violet (pastel)
        Error = '#FF6347'  # Tomato (keeping it close to red for visibility)
    }
    PredictionSource = 'History'
    PredictionViewStyle = 'ListView'
    BellStyle = 'None'
}
Set-PSReadLineOption @PSReadLineOptions

# Custom key handlers
Set-PSReadLineKeyHandler -Key UpArrow -Function HistorySearchBackward
Set-PSReadLineKeyHandler -Key DownArrow -Function HistorySearchForward
Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete
Set-PSReadLineKeyHandler -Chord 'Ctrl+d' -Function DeleteChar
Set-PSReadLineKeyHandler -Chord 'Ctrl+w' -Function BackwardDeleteWord
Set-PSReadLineKeyHandler -Chord 'Alt+d' -Function DeleteWord
Set-PSReadLineKeyHandler -Chord 'Ctrl+LeftArrow' -Function BackwardWord
Set-PSReadLineKeyHandler -Chord 'Ctrl+RightArrow' -Function ForwardWord
Set-PSReadLineKeyHandler -Chord 'Ctrl+z' -Function Undo
Set-PSReadLineKeyHandler -Chord 'Ctrl+y' -Function Redo

# Custom functions for PSReadLine
Set-PSReadLineOption -AddToHistoryHandler {
    param($line)
    $sensitive = @('password', 'secret', 'token', 'apikey', 'connectionstring')
    $hasSensitive = $sensitive | Where-Object { $line -match $_ }
    return ($null -eq $hasSensitive)
}

function Set-PredictionSource {
    # If function "Set-PredictionSource_Override" is defined in profile.ps1 file
    # then call it instead.
    if (Get-Command -Name "Set-PredictionSource_Override" -ErrorAction SilentlyContinue) {
        Set-PredictionSource_Override;
    } else {
        # Improved prediction settings
        Set-PSReadLineOption -PredictionSource HistoryAndPlugin
        Set-PSReadLineOption -MaximumHistoryCount 10000
    }
}
Set-PredictionSource

# Custom completion for common commands
$scriptblock = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $customCompletions = @{
        'git' = @('status', 'add', 'commit', 'push', 'pull', 'clone', 'checkout')
        'npm' = @('install', 'start', 'run', 'test', 'build')
        'deno' = @('run', 'compile', 'bundle', 'test', 'lint', 'fmt', 'cache', 'info', 'doc', 'upgrade')
    }

    $command = $commandAst.CommandElements[0].Value
    if ($customCompletions.ContainsKey($command)) {
        $customCompletions[$command] | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
}
Register-ArgumentCompleter -Native -CommandName git, npm, deno -ScriptBlock $scriptblock

$scriptblock = {
    param($wordToComplete, $commandAst, $cursorPosition)
    dotnet complete --position $cursorPosition $commandAst.ToString() |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
}
Register-ArgumentCompleter -Native -CommandName dotnet -ScriptBlock $scriptblock

#if (Get-Command -Name "Get-Theme_Override" -ErrorAction SilentlyContinue){
#    Get-Theme_Override;
#} else {
#    oh-my-posh init pwsh --config https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/cobalt2.omp.json | Invoke-Expression
#}

if (Get-Command zoxide -ErrorAction SilentlyContinue) {
    Invoke-Expression (& { (zoxide init --cmd z powershell | Out-String) })
} else {
    Write-Host "zoxide command not found. Attempting to install via winget..."
    try {
        winget install -e --id ajeetdsouza.zoxide
        Write-Host "zoxide installed successfully. Initializing..."
        Invoke-Expression (& { (zoxide init --cmd z powershell | Out-String) })
    } catch {
        Write-Error "Failed to install zoxide. Error: $_"
    }
}

## Un comment command below if the module is already installed (Terminal-icon, Oh-my-posh, Chocolatey)
## Add icon
#Import-Module Terminal-Icons

## Final Line to set prompt
oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH/kali.omp.json" | Invoke-Expression

#f45873b3-b655-43a6-b217-97c00aa0db58 PowerToys CommandNotFound module
Import-Module -Name Microsoft.WinGet.CommandNotFound

#function Get-Theme_Override {
#    oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH/kali.omp.json" | Invoke-Expression
#}

#f45873b3-b655-43a6-b217-97c00aa0db58
# Invoke express for vfox(Version-fox) SDK managerment for POSH (PowerShell)
Invoke-Expression "$(vfox activate pwsh)"
