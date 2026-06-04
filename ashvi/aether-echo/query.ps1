$vault = New-Object Windows.Security.Credentials.PasswordVault
$creds = $vault.RetrieveAll()
$cred = $creds | Where-Object { $_.Resource -eq 'aether-echo' -and $_.UserName -eq 'groq-api-key' }
if ($cred) {
    $cred.RetrievePassword()
    $key = $cred.Password
    $headers = @{
        "Authorization" = "Bearer $key"
    }
    $response = Invoke-RestMethod -Uri "https://api.groq.com/openai/v1/models" -Headers $headers
    $response.data | Select-Object id
} else {
    Write-Host "API key credential not found in Credential Manager"
}
