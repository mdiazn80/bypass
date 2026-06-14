# Release signing secrets

The release workflow (`.github/workflows/version-tag-and-binary.yml`) signs the
Tauri updater artifacts and produces a signed + notarized macOS build. It reads
these secrets from the repository's GitHub Actions secrets:

| Secret | Used for | Where it comes from |
| --- | --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Signs updater artifacts (all platforms) | Tauri signer key pair (generated locally) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password that protects the private key | Chosen when generating the key |
| `APPLE_CERTIFICATE` | Code-signing certificate (`.p12`, Base64) | Apple "Developer ID Application" certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password of the exported `.p12` | Chosen when exporting from Keychain |
| `APPLE_SIGNING_IDENTITY` | Identity Tauri uses to sign | The certificate's common name |
| `APPLE_ID` | Notarization account | Your Apple ID email |
| `APPLE_PASSWORD` | Notarization auth | App-specific password from appleid.apple.com |
| `APPLE_TEAM_ID` | Notarization team | Your Apple Developer Team ID (10 chars) |

> All values are pasted into **GitHub → repo → Settings → Secrets and variables
> → Actions → New repository secret**. The secret name must match exactly.
> Paste each value as a single line: the helper
> `scripts/normalize-tauri-signing-env.sh` strips stray `\r`/`\n`, but it is
> safest to avoid trailing newlines.

---

## 1. Updater signing key pair (`TAURI_SIGNING_PRIVATE_KEY*`)

Tauri's updater verifies every release with a public/private key pair. The
**public** key lives in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`)
and ships inside the app; the **private** key signs the artifacts in CI.

### 1.1 Generate the key pair

Run locally (from the repo root):

```bash
pnpm tauri signer generate -w ~/.tauri/bypass.key
```

- You will be prompted for a password. **Remember it** — that value becomes
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. (You can pass `-p "<password>"` instead
  of being prompted, or leave it empty for no password.)
- It writes two files:
  - `~/.tauri/bypass.key` — the **private** key (Base64).
  - `~/.tauri/bypass.key.pub` — the **public** key (Base64).

### 1.2 Fill the secrets

```bash
# TAURI_SIGNING_PRIVATE_KEY  -> full content of the private key file
cat ~/.tauri/bypass.key

# TAURI_SIGNING_PRIVATE_KEY_PASSWORD -> the password you chose above
```

Copy the entire output of the private key file into the
`TAURI_SIGNING_PRIVATE_KEY` secret.

### 1.3 Update the public key in the app

Put the public key into `src-tauri/tauri.conf.json`:

```bash
cat ~/.tauri/bypass.key.pub
```

```json
{
  "plugins": {
    "updater": {
      "pubkey": "<contents of bypass.key.pub>"
    }
  }
}
```

> **Key rotation warning:** the `pubkey` must correspond to the private key used
> for signing. If you generate a **new** key pair, you must update `pubkey` and
> release a new version. Clients that already have the **old** public key will
> reject updates signed with the new key, so plan rotations carefully.

Keep `~/.tauri/bypass.key` private (do **not** commit it).

---

## 2. Apple Developer prerequisites

You need a paid **Apple Developer Program** membership. For distribution
**outside** the Mac App Store (which is what this project does), the certificate
must be a **Developer ID Application** certificate — an "Apple Development"
certificate cannot notarize for distribution.

---

## 3. `APPLE_TEAM_ID`

Your 10-character Team ID.

- Find it at <https://developer.apple.com/account> → **Membership details** →
  *Team ID*, or
- Run `security find-identity -v -p codesigning` (the ID appears in parentheses
  in the identity name, see step 5).

Example: `AB12CD34EF`.

---

## 4. `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`

These come from a **Developer ID Application** certificate exported as a `.p12`.

### 4.1 Create the certificate (once)

1. On macOS, open **Keychain Access → Certificate Assistant → Request a
   Certificate From a Certificate Authority…** and save a CSR to disk.
2. Go to <https://developer.apple.com/account/resources/certificates/list>,
   click **+**, choose **Developer ID Application**, upload the CSR, and
   download the resulting `.cer`.
3. Double-click the `.cer` to import it into your login Keychain.

### 4.2 Export the `.p12`

1. In **Keychain Access**, find the certificate and click its disclosure triangle.
   You must see a **private key** nested underneath — if you don't, the `.p12`
   export option will be greyed out (see troubleshooting below).
2. Select the **certificate row** (not the private key row).
3. Right-click → **Export…** → format **Personal Information Exchange (.p12)**.
4. Set a password when prompted. **That password is `APPLE_CERTIFICATE_PASSWORD`.**

> **Troubleshooting: `.p12` export is greyed out**
>
> This happens when the private key is missing from the Keychain. The private key
> is only created on the Mac where the CSR was originally generated. Three paths:
>
> - **Same Mac, key present but grey**: make sure you selected the certificate
>   row, not the private-key row.
> - **Key missing, original Mac available**: export the `.p12` from that machine
>   and copy it here.
> - **Key missing, original Mac unavailable**: revoke the certificate at
>   [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates),
>   then repeat from step 4.1 on the machine you want to use. Creating a new CSR
>   generates a new private key in that machine's Keychain, which makes `.p12`
>   export possible.

### 4.3 Base64-encode for the secret

`APPLE_CERTIFICATE` is the `.p12` encoded as Base64 (single line):

```bash
base64 -i certificate.p12 | pbcopy      # copies to clipboard
# or write to a file:
base64 -i certificate.p12 -o cert.b64.txt
```

Paste the Base64 string into `APPLE_CERTIFICATE`.

---

## 5. `APPLE_SIGNING_IDENTITY`

This is the certificate's full common name. List the available identities:

```bash
security find-identity -v -p codesigning
```

You will see a line like:

```
1) ABCDEF0123... "Developer ID Application: Your Name (AB12CD34EF)"
```

`APPLE_SIGNING_IDENTITY` is the quoted string **without** the quotes:

```
Developer ID Application: Your Name (AB12CD34EF)
```

It must match the certificate inside `APPLE_CERTIFICATE`, otherwise signing
fails.

---

## 6. `APPLE_ID` and `APPLE_PASSWORD`

Used by `notarytool` to notarize the build.

- `APPLE_ID`: the email of the Apple ID that belongs to the developer team.
- `APPLE_PASSWORD`: an **app-specific password** (not your normal password).
  Generate it at <https://appleid.apple.com> → **Sign-In and Security →
  App-Specific Passwords → Generate**. Copy the value shown
  (format `abcd-efgh-ijkl-mnop`).

---

## 7. Add everything to GitHub

1. Go to **repo → Settings → Secrets and variables → Actions**.
2. For each name in the table at the top, click **New repository secret** and
   paste the value.
3. Re-run / trigger the release workflow.

### Local builds without secrets

You don't need any of these for day-to-day development or unsigned local
packages:

```bash
task build:local   # builds without signing or updater artifacts
```
