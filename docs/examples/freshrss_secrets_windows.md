# FreshRSS Automatic Login with GPG (Windows / PowerShell)

This guide shows how to securely store your FreshRSS credentials using GPG on Windows and use them with eilmeldung's `cmd:` secret feature.

## Install and Setup GPG

Install gpg4win via scoop:

```powershell
# Install gpg but don't run it yet (we need to create the env var first)
scoop bucket add extras
scoop install gpg4win
```

> **Important:** If you have `git` installed via scoop, it ships its own MinGW-based `gpg` that shadows gpg4win and cannot handle Windows-style paths. Override the shims to use gpg4win's native Windows binary:
>
> ```powershell
> scoop shim add gpg "$env:USERPROFILE\scoop\apps\gpg4win\current\GnuPG\bin\gpg.exe"
> scoop shim add gpg-agent "$env:USERPROFILE\scoop\apps\gpg4win\current\GnuPG\bin\gpg-agent.exe"
> scoop shim add gpgconf "$env:USERPROFILE\scoop\apps\gpg4win\current\GnuPG\bin\gpgconf.exe"
> scoop shim add gpg-connect-agent "$env:USERPROFILE\scoop\apps\gpg4win\current\GnuPG\bin\gpg-connect-agent.exe"
> ```
>
> Verify the correct binary is active:
> ```powershell
> gpg --version  # should mention GnuPG from gpg4win, not Git bundled gpg
> ```

Set up the GnuPG home directory and generate a key:

```powershell
# Create the GnuPG config directory
mkdir "$env:USERPROFILE\.config\gnupg" -Force

# Set permissions
icacls "$env:USERPROFILE\.config\gnupg" /inheritance:r /grant:r "$env:USERNAME`:F"

# Set GNUPGHOME permanently (gpg4win understands Windows paths)
[Environment]::SetEnvironmentVariable("GNUPGHOME", "$env:USERPROFILE\.config\gnupg", "User")

# Restart your terminal, then generate a key
gpg --gen-key
```

## Encrypt Your FreshRSS URL, Username, and Password

```powershell
mkdir "$env:USERPROFILE\.passwords" -Force
"https://your_freshrss_domain.com" | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-url.gpg"
"your_freshrss_username"           | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-user.gpg"
"your_api_password"                | gpg --encrypt --recipient "your_email_address" --output "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg"
```

### Test Decryption

```powershell
gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-url.gpg"
gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-user.gpg"
gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg"
```

Each command should print the decrypted value with no errors.

## Create the Retrieval Scripts

```powershell
New-Item "$env:USERPROFILE\.config\eilmeldung" -ItemType Directory -Force

Set-Content "$env:USERPROFILE\.config\eilmeldung\get-url.ps1"  'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-url.gpg"'
Set-Content "$env:USERPROFILE\.config\eilmeldung\get-user.ps1" 'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-user.gpg"'
Set-Content "$env:USERPROFILE\.config\eilmeldung\get-pass.ps1" 'gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg"'
```

### Test the Scripts

```powershell
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-url.ps1" '(gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-url.gpg").Trim()'
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-user.ps1" '(gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-user.gpg").Trim()'
Add-Content "$env:USERPROFILE\.config\eilmeldung\get-pass.ps1" '(gpg --quiet --decrypt "$env:USERPROFILE\.passwords\eilmeldung-pass.gpg").Trim()'
```

Each script should print the decrypted value.

## eilmeldung Config for FreshRSS

The `url`, `user`, and `password` fields all support the `cmd:` prefix on Windows.
Use `pwsh -NoProfile -File` to invoke your `.ps1` scripts, and `${USERPROFILE}` to
reference your home directory (expanded automatically by eilmeldung):

```toml
[login_setup]
login_type = "direct_password"
provider = "freshrss"
url      = "cmd:pwsh -NoProfile -File ${USERPROFILE}/.config/eilmeldung/get-url.ps1"
user     = "cmd:pwsh -NoProfile -File ${USERPROFILE}/.config/eilmeldung/get-user.ps1"
password = "cmd:pwsh -NoProfile -File ${USERPROFILE}/.config/eilmeldung/get-pass.ps1"
```
