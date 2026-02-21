# Automated Moltbook claim email sender
# Retries POST /api/v1/agents/verify-email every 30 seconds until success
# Rate limit resets at 2026-02-21T17:41:48 UTC

$claimToken = "moltbook_claim_UhwnzqM3ns6jsmNAZw01aKGjpvcLrr26"
$email = "kevanbtc@gmail.com"
$username = "Genesix"

$bodyFile = "$env:TEMP\moltbook-verify.json"
$body = @{
    claim_token = $claimToken
    email = $email
    username = $username
} | ConvertTo-Json -Compress
[System.IO.File]::WriteAllText($bodyFile, $body)

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Claim retry loop started" -ForegroundColor Cyan
Write-Host "  Token: $claimToken"
Write-Host "  Email: $email"
Write-Host "  Username: $username"
Write-Host ""

$maxAttempts = 30
for ($i = 1; $i -le $maxAttempts; $i++) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Attempt $i/$maxAttempts..." -NoNewline

    $result = curl.exe -s -w "`n%{http_code}" -X POST `
        "https://www.moltbook.com/api/v1/agents/verify-email" `
        -H "Content-Type: application/json" `
        -d "@$bodyFile" 2>&1

    $lines = $result -split "`n"
    $httpCode = $lines[-1].Trim()
    $responseBody = ($lines[0..($lines.Length-2)] -join "`n").Trim()

    if ($httpCode -eq "200" -or $httpCode -eq "201") {
        Write-Host " SUCCESS!" -ForegroundColor Green
        Write-Host ""
        Write-Host "=== VERIFICATION EMAIL SENT ===" -ForegroundColor Green
        Write-Host $responseBody
        Write-Host ""
        Write-Host "CHECK YOUR EMAIL: $email" -ForegroundColor Yellow
        Write-Host "Click the magic link within 15 minutes!" -ForegroundColor Yellow
        Remove-Item $bodyFile -ErrorAction SilentlyContinue
        exit 0
    }
    elseif ($httpCode -eq "429") {
        # Parse retry_after from response
        try {
            $json = $responseBody | ConvertFrom-Json
            $retryAfter = [int]$json.retry_after_seconds
            $resetAt = $json.reset_at
            Write-Host " rate limited (resets: $resetAt, ${retryAfter}s)" -ForegroundColor Yellow
            
            # Wait the minimum of retry_after or 30 seconds
            $waitTime = [Math]::Min($retryAfter + 2, 60)
            Write-Host "  Waiting ${waitTime}s..."
            Start-Sleep -Seconds $waitTime
        } catch {
            Write-Host " rate limited (waiting 30s)" -ForegroundColor Yellow
            Start-Sleep -Seconds 30
        }
    }
    else {
        Write-Host " HTTP $httpCode" -ForegroundColor Red
        Write-Host "  $responseBody"
        
        if ($httpCode -eq "400") {
            Write-Host ""
            Write-Host "Bad request - check parameters. Stopping." -ForegroundColor Red
            Remove-Item $bodyFile -ErrorAction SilentlyContinue
            exit 1
        }
        # For 500s, retry after a short wait
        Start-Sleep -Seconds 10
    }
}

Write-Host ""
Write-Host "Max attempts reached. Try manually later." -ForegroundColor Red
Remove-Item $bodyFile -ErrorAction SilentlyContinue
exit 1
