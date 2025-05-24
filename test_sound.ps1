# Test Windows Audio Playback
# This script uses PowerShell to play a sound through the Windows API

# Define a function to play sound using System.Media.SoundPlayer
function Test-SoundPlayback {
    Write-Host "🔊 Testing Windows Sound Playback..."
    
    # Test with WAV file
    $wavFile = Join-Path (Get-Location) "sounds\wav\beep.wav"
    
    if (Test-Path $wavFile) {
        try {
            Write-Host "✅ Found WAV file: $wavFile"
            
            # Create a SoundPlayer object
            Add-Type -AssemblyName System.Windows.Forms
            $player = New-Object System.Media.SoundPlayer
            $player.SoundLocation = $wavFile
            
            # Try to play the sound
            Write-Host "🎵 Playing WAV file..."
            $player.Play()
            
            # Wait a moment for the sound to play
            Start-Sleep -Seconds 2
            
            Write-Host "✅ Sound playback attempt completed."
            Write-Host "Did you hear the sound? (Y/N)"
        }
        catch {
            Write-Host "❌ Error playing sound: $_"
        }
    }
    else {
        Write-Host "❌ WAV file not found: $wavFile"
    }
    
    # Check system volume
    Write-Host "`n📊 System Audio Information:"
    Write-Host "- Checking if audio is muted..."
    
    try {
        # This requires the AudioDeviceCmdlets module
        # If not found, provide instructions
        if (-not (Get-Command Get-AudioDevice -ErrorAction SilentlyContinue)) {
            Write-Host "⚠️ AudioDeviceCmdlets module not found."
            Write-Host "For more detailed audio diagnostics, you can install it with:"
            Write-Host "Install-Module -Name AudioDeviceCmdlets"
        }
        else {
            $audioDevice = Get-AudioDevice -Playback
            Write-Host "- Playback Device: $($audioDevice.Name)"
            Write-Host "- Volume: $($audioDevice.Volume)%"
            Write-Host "- Muted: $($audioDevice.Muted)"
        }
    }
    catch {
        Write-Host "❌ Error checking audio settings: $_"
    }
    
    # Try another approach - use Windows media player COM object
    Write-Host "`n🎵 Trying alternative playback method..."
    try {
        $wmp = New-Object -ComObject WMPlayer.OCX
        $wmp.settings.volume = 100
        $wmp.URL = $wavFile
        Write-Host "✅ Playing through WMP - please wait..."
        Start-Sleep -Seconds 3
        $wmp.close()
    }
    catch {
        Write-Host "❌ Error with Windows Media Player: $_"
    }
    
    Write-Host "`n🔍 Sound playback test complete."
}

# Run the test
Test-SoundPlayback
