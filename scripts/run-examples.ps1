<#
.SYNOPSIS
Primerのexampleをまとめて実行します。

.PARAMETER Pattern
実行するファイル名のpatternです。既定値は*.primです。

.PARAMETER SkipBuild
実行前のcargo buildを省略します。
#>
[CmdletBinding()]
param(
    [string]$Pattern = "*.prim",
    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$utf8 = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = $utf8
$OutputEncoding = $utf8

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$examplesDirectory = Join-Path $repositoryRoot "examples"
$primer = Join-Path $repositoryRoot "target\debug\primer.exe"
$examples = @(
    Get-ChildItem -LiteralPath $examplesDirectory -Filter $Pattern -File |
        Sort-Object Name
)

if ($examples.Count -eq 0) {
    Write-Host "[ERROR] pattern '$Pattern' に一致するexampleがありません。" -ForegroundColor Red
    exit 1
}

Push-Location $repositoryRoot
try {
    if (-not $SkipBuild) {
        Write-Host "Primerをbuildしています..."
        & cargo build --quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] cargo buildに失敗しました。" -ForegroundColor Red
            exit 1
        }
    }

    if (-not (Test-Path -LiteralPath $primer -PathType Leaf)) {
        Write-Host "[ERROR] Primer実行ファイルがありません。-SkipBuildを外して再実行してください。" -ForegroundColor Red
        exit 1
    }

    $passed = 0
    $failed = [System.Collections.Generic.List[string]]::new()

    foreach ($example in $examples) {
        Write-Host ""
        Write-Host "=== $($example.Name) ==="
        & $primer run $example.FullName

        if ($LASTEXITCODE -eq 0) {
            $passed += 1
            Write-Host "[PASS] $($example.Name)"
        }
        else {
            $failed.Add($example.Name)
            Write-Host "[FAIL] $($example.Name)" -ForegroundColor Red
        }
    }

    Write-Host ""
    Write-Host "=== Summary ==="
    Write-Host "成功: $passed"
    Write-Host "失敗: $($failed.Count)"

    if ($failed.Count -gt 0) {
        Write-Host "失敗したexample: $($failed -join ', ')" -ForegroundColor Red
        exit 1
    }
}
finally {
    Pop-Location
}
