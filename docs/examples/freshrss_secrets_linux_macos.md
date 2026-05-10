# FreshRSS Automatic Login with pass (Linux and macOS)

This guide shows how to securely store your FreshRSS credentials using [`pass`](https://www.passwordstore.org/), the standard Unix password manager, and use them with eilmeldung's `cmd:` secret feature.

## Install and Setup pass

Install `pass` via your package manager:

```bash
# Debian/Ubuntu
apt install pass

# macOS
brew install pass
```

Generate a GPG key if you don't have one, then initialize `pass` with it:

```bash
gpg --gen-key
pass init "your_email_address"
```

## Store Your FreshRSS Credentials

```bash
pass insert eilmeldung/url
pass insert eilmeldung/user
pass insert eilmeldung/password
```

## eilmeldung Config for FreshRSS

```toml
[login_setup]
login_type = "direct_password"
provider = "freshrss"
url = "cmd:pass eilmeldung/url"
user = "cmd:pass eilmeldung/user"
password = "cmd:pass eilmeldung/password"
```
