param(
    [switch]$ListPackages
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

$packages = @(
    @{ Name = "duan"; Path = "packages/duan"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "duan-runtime"; Path = "packages/duan-core"; TestArgs = @("test", "--lib", "--tests", "--all-features"); Doc = $true },
    @{ Name = "duan-catalog"; Path = "packages/duan-catalog"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "duan-scenario"; Path = "packages/duan-scenario"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "duan-runner"; Path = "packages/duan-runner"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "duan-build"; Path = "packages/duan-build"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "duan-cli"; Path = "packages/duan-cli"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "duan-macros"; Path = "packages/duan-macros"; TestArgs = @("test", "--all-targets", "--all-features"); Doc = $true },
    @{ Name = "example-free-fall-body"; Path = "examples/packages/free-fall/body"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-free-fall-gravity"; Path = "examples/packages/free-fall/gravity"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-free-fall-scene-objects"; Path = "examples/packages/free-fall/scene-objects"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-naval-combat-engagement"; Path = "examples/packages/naval-combat/engagement"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-naval-combat-fleet-objects"; Path = "examples/packages/naval-combat/fleet-objects"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-naval-combat-maneuver"; Path = "examples/packages/naval-combat/maneuver"; TestArgs = @("test", "--all-targets", "--all-features") },
    @{ Name = "example-naval-combat-platform"; Path = "examples/packages/naval-combat/platform"; TestArgs = @("test", "--all-targets", "--all-features") }
)

if ($ListPackages) {
    foreach ($package in $packages) {
        Write-Output "$($package.Name) $($package.Path)"
    }
    exit 0
}

function Invoke-Cargo {
    param(
        [Parameter(Mandatory = $true)]
        [string]$PackageName,
        [Parameter(Mandatory = $true)]
        [string]$PackagePath,
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    Write-Host ""
    Write-Host "==> $PackageName :: cargo $($Arguments -join ' ')"
    Push-Location $PackagePath
    try {
        & cargo @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "cargo $($Arguments -join ' ') failed in $PackagePath with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

foreach ($package in $packages) {
    $packagePath = Join-Path $repoRoot $package.Path
    if (-not (Test-Path (Join-Path $packagePath "Cargo.toml"))) {
        throw "Missing Cargo.toml for $($package.Name) at $($package.Path)"
    }

    Invoke-Cargo $package.Name $packagePath @("fmt", "--all", "--", "--check")
    Invoke-Cargo $package.Name $packagePath @("clippy", "--all-targets", "--all-features", "--", "-D", "warnings")
    Invoke-Cargo $package.Name $packagePath $package.TestArgs
    Invoke-Cargo $package.Name $packagePath @("build", "--all-targets", "--all-features")
    if ($package.Doc) {
        Invoke-Cargo $package.Name $packagePath @("doc", "--all-features", "--no-deps")
    }
}
