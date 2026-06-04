# FreshRSS Automatic Login with GPG (Windows / PowerShell)

This guide shows how to securely store your FreshRSS credentials using GPG on Windows and use them with eilmeldung's `cmd:` secret feature.

## Install and Setup GPG

```powershell
# Install gpg but don't run it yet (we need to create the env var first)
scoop bucket add extras
scoop install gpg4win

# Create the GnuPG config directory
mkdir "$env:USERPROFILE\.config\gnupg" -Force

# Set permissions
icacls "$env:USERPROFILE\.config\gnupg" /inheritance:r /grant:r "$env:USERNAME`:F"

# Set the GNUPGHOME env var permanently
[Environment]::SetEnvironmentVariable("GNUPGHOME", "$env:USERPROFILE\.config\gnupg", "User")

# Restart your terminal, then generate a key
~\scoop\apps\gpg4win\current\GnuPG\bin\gpg.exe --gen-key
```

## Encrypt Your FreshRSS URL, Username, and Password

```powershell
mkdir ~/.passwords
"https://your_freshrss_domain.com" | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-url.gpg"
"your_freshrss_username" | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-user.gpg"
"your_api_password" | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg"
```

### Test Decryption

```powershell
gpg --quiet --decrypt ~/.passwords/eilmeldung-url.gpg
gpg --quiet --decrypt ~/.passwords/eilmeldung-user.gpg
gpg --quiet --decrypt ~/.passwords/eilmeldung-pass.gpg
```

## Create the Retrieval Scripts

```powershell
New-Item "$env:USERPROFILE\.config\eilmeldung\get-url.ps1" -Force
New-Item "$env:USERPROFILE\.config\eilmeldung\get-user.ps1" -Force
New-Item "$env:USERPROFILE\.config\eilmeldung\get-pass.ps1" -Force
```

Add content to each:

```powershell
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-url.ps1" 'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-url.gpg"'
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-user.ps1" 'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-user.gpg"'
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-pass.ps1" 'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg"'
```

## eilmeldung Config for FreshRSS

```toml
[login_setup]
login_type = "direct_password"
provider = "freshrss"
url = "cmd:%USERPROFILE%/.config/eilmeldung/get-url.ps1"
user = "cmd:%USERPROFILE%/.config/eilmeldung/get-user.ps1"
password = "cmd:%USERPROFILE%/.config/eilmeldung/get-pass.ps1"
```
