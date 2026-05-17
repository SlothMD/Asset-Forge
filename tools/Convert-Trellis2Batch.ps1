param(
    [Parameter(Mandatory = $true)]
    [string] $SourceDir,

    [Parameter(Mandatory = $true)]
    [string] $FinalOutputDir,

    [string] $WorkflowPath = "D:\StableDiffusion\Workflows\geometry_texture.json",
    [string] $ComfyUrl = "http://127.0.0.1:8000",
    [string] $ComfyOutputDir = "C:\Users\sloth\Documents\ComfyUI\output",
    [string] $StagingDir = "",
    [string] $OrientationCsv = "",
    [string] $ClientId = "asset_forge_trellis2_batch",
    [string] $OutputSuffix = "",
    [int] $Limit = 0,
    [int] $TargetFaceCount = 500000,
    [int] $TextureSize = 2048,
    [switch] $Force,
    [switch] $WhatIfOnly
)

$ErrorActionPreference = "Stop"

function Convert-ToSlug {
    param([string] $Value)

    $slug = $Value.ToLowerInvariant() -replace '[^a-z0-9]+', '-'
    return $slug.Trim('-')
}

function Read-OrientationMap {
    param([string] $Path)

    $map = @{}
    if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
        return $map
    }

    if ([System.IO.Path]::GetExtension($Path).Equals(".json", [System.StringComparison]::OrdinalIgnoreCase)) {
        $manifest = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
        if ($manifest.assets) {
            foreach ($property in $manifest.assets.PSObject.Properties) {
                $asset = $property.Value
                if (-not $asset.fileName -or -not $asset.transform) {
                    continue
                }

                $orientation = [pscustomobject]@{
                    RotateDegrees = if ($asset.transform.rotateDegrees) { [double]$asset.transform.rotateDegrees } else { 0 }
                    FlipHorizontal = [bool]$asset.transform.flipHorizontal
                    FlipVertical = [bool]$asset.transform.flipVertical
                }
                $map[$asset.fileName] = $orientation
                if ($asset.relativePath) {
                    $map[$asset.relativePath] = $orientation
                }
            }
        }

        return $map
    }

    Import-Csv -LiteralPath $Path | ForEach-Object {
        if (-not $_.fileName) {
            return
        }

        $map[$_.fileName] = [pscustomobject]@{
            RotateDegrees = if ($_.rotateDegrees) { [double]$_.rotateDegrees } else { 0 }
            FlipHorizontal = "$($_.flipHorizontal)".Equals("true", [System.StringComparison]::OrdinalIgnoreCase)
            FlipVertical = "$($_.flipVertical)".Equals("true", [System.StringComparison]::OrdinalIgnoreCase)
        }
    }

    return $map
}

function Get-RotateFlipType {
    param(
        [int] $RotateDegrees,
        [bool] $FlipHorizontal,
        [bool] $FlipVertical
    )

    $normalized = (($RotateDegrees % 360) + 360) % 360
    $flip = if ($FlipHorizontal -and $FlipVertical) {
        "FlipXY"
    }
    elseif ($FlipHorizontal) {
        "FlipX"
    }
    elseif ($FlipVertical) {
        "FlipY"
    }
    else {
        "FlipNone"
    }

    $enumName = "Rotate$($normalized)$flip"
    return [System.Enum]::Parse([System.Drawing.RotateFlipType], $enumName)
}

function Test-IsRightAngleRotation {
    param([double] $RotateDegrees)

    $normalized = (($RotateDegrees % 360) + 360) % 360
    return [Math]::Abs($normalized % 90) -lt 0.001
}

function New-OrientedImage {
    param(
        [System.IO.FileInfo] $SourceFile,
        [object] $Orientation,
        [string] $TargetDir
    )

    if (-not $Orientation -or (($Orientation.RotateDegrees -eq 0) -and -not $Orientation.FlipHorizontal -and -not $Orientation.FlipVertical)) {
        return $SourceFile.FullName
    }

    Add-Type -AssemblyName System.Drawing

    if (-not (Test-Path -LiteralPath $TargetDir)) {
        New-Item -ItemType Directory -Path $TargetDir | Out-Null
    }

    $targetPath = Join-Path $TargetDir $SourceFile.Name
    $image = [System.Drawing.Image]::FromFile($SourceFile.FullName)
    try {
        if (Test-IsRightAngleRotation -RotateDegrees $Orientation.RotateDegrees) {
            $image.RotateFlip((Get-RotateFlipType `
                -RotateDegrees $Orientation.RotateDegrees `
                -FlipHorizontal $Orientation.FlipHorizontal `
                -FlipVertical $Orientation.FlipVertical))
            $image.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        else {
            $canvas = New-Object System.Drawing.Bitmap $image.Width, $image.Height, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
            $graphics = [System.Drawing.Graphics]::FromImage($canvas)
            try {
                $graphics.Clear([System.Drawing.Color]::Transparent)
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
                $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
                $graphics.TranslateTransform($image.Width / 2.0, $image.Height / 2.0)
                $scaleX = if ($Orientation.FlipHorizontal) { -1 } else { 1 }
                $scaleY = if ($Orientation.FlipVertical) { -1 } else { 1 }
                $graphics.ScaleTransform($scaleX, $scaleY)
                $graphics.RotateTransform($Orientation.RotateDegrees)
                $graphics.TranslateTransform(-$image.Width / 2.0, -$image.Height / 2.0)
                $graphics.DrawImage($image, 0, 0, $image.Width, $image.Height)
                $canvas.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
            }
            finally {
                $graphics.Dispose()
                $canvas.Dispose()
            }
        }
    }
    finally {
        $image.Dispose()
    }

    return $targetPath
}

function Upload-ComfyImage {
    param([string] $ImagePath)

    $form = @{
        image = Get-Item -LiteralPath $ImagePath
        type = "input"
        overwrite = "true"
    }

    $response = Invoke-RestMethod `
        -Uri "$ComfyUrl/upload/image" `
        -Method Post `
        -Form $form

    if ($response.name) {
        return [string]$response.name
    }

    return (Split-Path $ImagePath -Leaf)
}

function Submit-ComfyPrompt {
    param([object] $Workflow)

    $body = @{
        prompt = $Workflow
        client_id = $ClientId
    } | ConvertTo-Json -Depth 100

    $response = Invoke-RestMethod `
        -Uri "$ComfyUrl/prompt" `
        -Method Post `
        -ContentType "application/json" `
        -Body $body

    if (-not $response.prompt_id) {
        throw "ComfyUI did not return a prompt_id. Response was: $($response | ConvertTo-Json -Depth 20)"
    }

    return [string]$response.prompt_id
}

function Wait-ForPromptComplete {
    param(
        [string] $PromptId,
        [int] $PollSeconds = 5,
        [int] $TimeoutMinutes = 45
    )

    $deadline = (Get-Date).AddMinutes($TimeoutMinutes)

    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds $PollSeconds

        try {
            $history = Invoke-RestMethod `
                -Uri "$ComfyUrl/history/$PromptId" `
                -Method Get

            if ($history.PSObject.Properties.Name -contains $PromptId) {
                return
            }
        }
        catch {
            # Keep polling. Comfy may not have written history yet.
        }
    }

    throw "Timed out waiting for prompt to complete: $PromptId"
}

function Wait-ForOutputGlb {
    param(
        [string] $Root,
        [datetime] $StartedAt,
        [int] $PollSeconds = 5,
        [int] $TimeoutMinutes = 45
    )

    $deadline = (Get-Date).AddMinutes($TimeoutMinutes)

    while ((Get-Date) -lt $deadline) {
        $match = Get-ChildItem $ComfyOutputDir -Filter "$Root*.glb" -File -ErrorAction SilentlyContinue |
            Where-Object { $_.LastWriteTime -ge $StartedAt } |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1

        if ($match) {
            return $match.FullName
        }

        Start-Sleep -Seconds $PollSeconds
    }

    throw "Timed out waiting for GLB starting with '$Root' in $ComfyOutputDir"
}

if (-not (Test-Path -LiteralPath $WorkflowPath)) {
    throw "Workflow not found: $WorkflowPath"
}

if (-not (Test-Path -LiteralPath $SourceDir)) {
    throw "Source folder not found: $SourceDir"
}

if (-not (Test-Path -LiteralPath $FinalOutputDir)) {
    New-Item -ItemType Directory -Path $FinalOutputDir | Out-Null
}

if ([string]::IsNullOrWhiteSpace($StagingDir)) {
    $StagingDir = Join-Path $FinalOutputDir "_oriented-input"
}

$orientationMap = Read-OrientationMap -Path $OrientationCsv
$sourceFiles = Get-ChildItem -LiteralPath $SourceDir -Filter "*.png" -File | Sort-Object Name

if ($Limit -gt 0) {
    $sourceFiles = $sourceFiles | Select-Object -First $Limit
}

Write-Host "Found $($sourceFiles.Count) PNG files to check."
Write-Host "Output: $FinalOutputDir"

foreach ($sourceFile in $sourceFiles) {
    $root = Convert-ToSlug ([System.IO.Path]::GetFileNameWithoutExtension($sourceFile.Name))
    if (-not [string]::IsNullOrWhiteSpace($OutputSuffix)) {
        $root = "$root-$OutputSuffix"
    }

    $existing = Get-ChildItem -LiteralPath $FinalOutputDir -Filter "$root*.glb" -File -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if ($existing -and -not $Force) {
        Write-Host "Skipping $($sourceFile.Name), matching GLB already exists."
        continue
    }

    $relativeSourcePath = $sourceFile.FullName.Substring((Resolve-Path -LiteralPath $SourceDir).Path.Length).TrimStart('\', '/').Replace('\', '/')
    $orientation = $orientationMap[$relativeSourcePath]
    if (-not $orientation) {
        $orientation = $orientationMap[$sourceFile.Name]
    }
    $inputImage = New-OrientedImage -SourceFile $sourceFile -Orientation $orientation -TargetDir $StagingDir

    Write-Host ""
    Write-Host "Processing $($sourceFile.Name) -> $root.glb"
    if ($orientation) {
        Write-Host "Orientation: rotate $($orientation.RotateDegrees), flipH $($orientation.FlipHorizontal), flipV $($orientation.FlipVertical)"
    }

    if ($WhatIfOnly) {
        continue
    }

    $workflow = Get-Content -LiteralPath $WorkflowPath -Raw | ConvertFrom-Json
    $uploadedName = Upload-ComfyImage -ImagePath $inputImage

    $workflow.'1'.inputs.image = $uploadedName
    $workflow.'86'.inputs.filename_prefix = $root
    $workflow.'86'.inputs.file_format = "glb"

    if ($workflow.PSObject.Properties.Name -contains "97") {
        $workflow.'97'.inputs.target_face_count = $TargetFaceCount
    }
    if ($workflow.PSObject.Properties.Name -contains "98") {
        $workflow.'98'.inputs.texture_size = $TextureSize
    }

    $startedAt = Get-Date
    $promptId = Submit-ComfyPrompt -Workflow $workflow
    Write-Host "Queued prompt $promptId"

    Wait-ForPromptComplete -PromptId $promptId

    $generatedGlb = Wait-ForOutputGlb -Root $root -StartedAt $startedAt
    $finalPath = Join-Path $FinalOutputDir "$root.glb"

    if (Test-Path -LiteralPath $finalPath) {
        $stamp = Get-Date -Format "yyyyMMdd_HHmmss"
        $finalPath = Join-Path $FinalOutputDir "$root`_$stamp.glb"
    }

    Move-Item -LiteralPath $generatedGlb -Destination $finalPath
    Write-Host "Saved $finalPath"
}

Write-Host ""
Write-Host "Batch complete."
