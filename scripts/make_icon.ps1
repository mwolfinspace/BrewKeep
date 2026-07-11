# BrewKeep — Generate icon.ico from coffee emoji
# Uses .NET System.Drawing to render ☕ emoji on a canvas

Add-Type -AssemblyName System.Drawing

$size = 256
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
$g.Clear([System.Drawing.Color]::Transparent)

$font = New-Object System.Drawing.Font("Segoe UI Emoji", 220, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
$brush = [System.Drawing.Brushes]::White

# Measure and center
$textSize = $g.MeasureString([char]0x2615, $font)
$x = ($size - $textSize.Width) / 2
$y = ($size - $textSize.Height) / 2

$g.DrawString([char]0x2615, $font, $brush, $x, $y)

# Save as multi-resolution ICO
$icoPath = Join-Path $PSScriptRoot "..\icon.ico"
$pngPath = Join-Path $PSScriptRoot "..\icon_source.png"

$bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Convert to ICO using .NET
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $ms.ToArray()
$ms.Close()

# Build ICO manually
$icoStream = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($icoStream)

# ICO header
$writer.Write([uint16]0)       # Reserved
$writer.Write([uint16]1)       # Type: ICO
$writer.Write([uint16]1)       # Count: 1 image

# ICO directory entry (256x256)
$writer.Write([byte]0)         # Width (0 = 256)
$writer.Write([byte]0)         # Height (0 = 256)
$writer.Write([byte]0)         # Color palette
$writer.Write([byte]0)         # Reserved
$writer.Write([uint16]1)       # Color planes
$writer.Write([uint16]32)      # Bits per pixel
$writer.Write([uint32]$pngBytes.Length)  # Size of image data
$writer.Write([uint32]22)      # Offset to image data (6 + 16)

# PNG image data
$writer.Write($pngBytes)

$icoBytes = $icoStream.ToArray()
$writer.Close()
$icoStream.Close()

[System.IO.File]::WriteAllBytes($icoPath, $icoBytes)

Write-Host "Icon generated: $icoPath"
Write-Host "Source PNG: $pngPath"

$g.Dispose()
$bmp.Dispose()
