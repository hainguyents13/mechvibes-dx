#!/usr/bin/env pwsh
# Script to convert soundpack timing from single timing to separate keydown/keyup timing
# Formula:
# Split the original timing into two equal parts:
# keydown_start = original_start
# keydown_end = original_start + (original_end - original_start) / 2  (midpoint)
# keyup_start = original_start + (original_end - original_start) / 2   (midpoint)
# keyup_end = original_end

param(
    [Parameter(Mandatory=$true)]
    [string]$ConfigPath
)

if (-not (Test-Path $ConfigPath)) {
    Write-Error "Config file not found: $ConfigPath"
    exit 1
}

Write-Host "🔄 Converting timing format in: $ConfigPath" -ForegroundColor Cyan

# Read the JSON config
$jsonContent = Get-Content $ConfigPath -Raw | ConvertFrom-Json

# Check if already converted
$firstKey = ($jsonContent.definitions | Get-Member -MemberType NoteProperty | Select-Object -First 1).Name
if ($jsonContent.definitions.$firstKey.timing.Count -eq 2 -and $jsonContent.definitions.$firstKey.timing[0].Count -eq 2) {
    Write-Host "✅ Already converted to keydown/keyup format" -ForegroundColor Green
    return
}

Write-Host "📊 Converting timing entries..." -ForegroundColor Yellow

# Convert each definition's timing
foreach ($keyProperty in ($jsonContent.definitions | Get-Member -MemberType NoteProperty)) {
    $keyName = $keyProperty.Name
    $keyDef = $jsonContent.definitions.$keyName
    
    if ($keyDef.timing -and $keyDef.timing.Count -gt 0) {
        $originalStart = $keyDef.timing[0][0]
        $originalEnd = $keyDef.timing[0][1]
          # Apply the formula - split timing into two equal parts
        $duration = $originalEnd - $originalStart
        $midpoint = $originalStart + ($duration / 2)
        
        $keydownStart = $originalStart
        $keydownEnd = $midpoint
        $keyupStart = $midpoint  
        $keyupEnd = $originalEnd
        
        # Update timing with new format: [[keydown_start, keydown_end], [keyup_start, keyup_end]]
        $keyDef.timing = @(
            @($keydownStart, $keydownEnd),
            @($keyupStart, $keyupEnd)
        )
        
        Write-Host "  🔧 ${keyName}: [${originalStart}, ${originalEnd}] → [[${keydownStart}, ${keydownEnd}], [${keyupStart}, ${keyupEnd}]]" -ForegroundColor Gray
    }
}

# Update config version to indicate the change (keep as v2)
# $jsonContent.config_version = "2"

# Add a note about the timing format
if (-not $jsonContent.PSObject.Properties['timing_format']) {
    $jsonContent | Add-Member -MemberType NoteProperty -Name 'timing_format' -Value 'keydown_keyup_separate'
}

# Save back to file with proper formatting
$jsonContent | ConvertTo-Json -Depth 10 | Out-File $ConfigPath -Encoding UTF8

Write-Host "✅ Successfully converted timing format!" -ForegroundColor Green
Write-Host "📁 Updated: $ConfigPath" -ForegroundColor Cyan
