#!/usr/bin/env pwsh
# Batch convert all soundpack timing formats

Write-Host "🔄 Converting timing format for all soundpacks..." -ForegroundColor Cyan

# Set working directory to parent directory (project root)
Set-Location (Split-Path -Parent $PSScriptRoot)

# Find all config.json files in soundpacks directory
$configFiles = Get-ChildItem -Path "soundpacks" -Recurse -Name "config.json" | ForEach-Object { "soundpacks\$_" }

Write-Host "📋 Found $($configFiles.Count) soundpack configs:" -ForegroundColor Green
foreach ($file in $configFiles) {
    Write-Host "  📄 $file" -ForegroundColor Gray
}

Write-Host ""
Write-Host "🚀 Starting batch conversion..." -ForegroundColor Yellow

$converted = 0
$skipped = 0
$errors = 0

foreach ($configFile in $configFiles) {
    try {
        Write-Host "Processing: $configFile" -ForegroundColor Cyan        # Run the conversion script
        $result = & "utils\convert-timing.ps1" -ConfigPath $configFile 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            if ($result -match "Already converted") {
                Write-Host "  ⏭️  Already converted" -ForegroundColor Yellow
                $skipped++
            } else {
                Write-Host "  ✅ Successfully converted" -ForegroundColor Green
                $converted++
            }
        } else {
            Write-Host "  ❌ Error: $result" -ForegroundColor Red
            $errors++
        }
    } catch {
        Write-Host "  ❌ Exception: $($_.Exception.Message)" -ForegroundColor Red
        $errors++
    }
    
    Write-Host ""
}

Write-Host "🎉 Batch conversion completed!" -ForegroundColor Cyan
Write-Host "📊 Summary:" -ForegroundColor Green
Write-Host "  ✅ Converted: $converted" -ForegroundColor Green
Write-Host "  ⏭️  Skipped: $skipped" -ForegroundColor Yellow
Write-Host "  ❌ Errors: $errors" -ForegroundColor Red
