$ErrorActionPreference = 'Stop'
$root = 'c:\Program Files\Git\code\FactoryTesting\FactoryTesting\src-tauri'

# Collect target files
$targets = Get-ChildItem -Path "$root\src\bin" -Recurse -Filter *.rs
$targets += Get-ChildItem -Path "$root\src\domain\services\mocks" -Filter *.rs
$targets += Get-ChildItem -Path "$root\src\infrastructure\extra\infrastructure" -Recurse -Filter mock_*.rs

$extra = @(
    "$root\src\domain\services\mocks\test_data_generator.rs",
    "$root\src\domain\services\mocks\mock_persistence_service.rs"
)
foreach ($p in $extra) {
    if (Test-Path $p) { $targets += Get-Item $p }
}

# Prepend flag to each file if absent
foreach ($file in $targets) {
    $firstLine = Get-Content -Path $file.FullName -TotalCount 1 -ErrorAction Stop
    if ($firstLine -notmatch '^#!\[cfg\(FALSE\)\]') {
        $content = Get-Content -Path $file.FullName -Raw -ErrorAction Stop
        Set-Content -Path $file.FullName -Value ("#![cfg(FALSE)]`r`n" + $content)
    }
}

# Verify
$missing = @()
foreach ($file in $targets) {
    $firstLine = Get-Content -Path $file.FullName -TotalCount 1 -ErrorAction Stop
    if ($firstLine -notmatch '^#!\[cfg\(FALSE\)\]') { $missing += $file.FullName }
}

if ($missing.Count -eq 0) {
    Write-Host 'ALL_OK'
} else {
    Write-Host 'MISSING:'
    $missing | ForEach-Object { Write-Host $_ }
    exit 1
}
