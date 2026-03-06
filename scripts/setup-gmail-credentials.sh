#!/usr/bin/env zsh
# setup-gmail-credentials.sh — one-time setup for GPG-encrypted msmtp credentials.
#
# What this does:
#   1. Creates a GPG key (if none exists) for encrypting credentials.
#   2. Prompts for your Gmail address and App Password (never stored in plaintext).
#   3. Encrypts the App Password to ~/.msmtp-gmail.gpg.
#   4. Patches ~/.msmtprc with your Gmail address and passwordeval directive.
#
# Run once:  ./scripts/setup-gmail-credentials.sh

set -euo pipefail
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"

CREDS_FILE="$HOME/.msmtp-gmail.gpg"
MSMTPRC="$HOME/.msmtprc"

# ---------------------------------------------------------------------------
# 1. Prompt for Gmail address
# ---------------------------------------------------------------------------

print -n "Gmail address (FROM address for test reports): "
read -r GMAIL

if [[ -z "$GMAIL" ]]; then
    echo "Error: Gmail address is required." >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# 2. Create GPG key if none exists
# ---------------------------------------------------------------------------

existing_key="$(gpg --list-secret-keys "$GMAIL" 2>/dev/null || true)"

if [[ -z "$existing_key" ]]; then
    echo ""
    echo "No GPG key found for $GMAIL — generating one now."
    echo "You will be prompted to set a passphrase for the key."
    echo "(Use a strong passphrase and save it in your password manager.)"
    echo ""
    gpg --batch --gen-key <<EOF
%no-protection
Key-Type: EdDSA
Key-Curve: ed25519
Subkey-Type: ECDH
Subkey-Curve: cv25519
Name-Real: NyxForge CI
Name-Email: ${GMAIL}
Expire-Date: 0
%commit
EOF
    echo ""
    echo "GPG key created."
else
    echo "GPG key already exists for $GMAIL."
fi

# ---------------------------------------------------------------------------
# 3. Encrypt App Password
# ---------------------------------------------------------------------------

echo ""
print -n "App Password (input hidden): "
read -rs APP_PASSWORD
echo ""

if [[ -z "$APP_PASSWORD" ]]; then
    echo "Error: App Password is required." >&2
    exit 1
fi

# Write to a temp file, encrypt, then securely delete it.
TMPFILE="$(mktemp)"
printf '%s' "$APP_PASSWORD" > "$TMPFILE"
gpg --quiet --batch --yes \
    --recipient "$GMAIL" \
    --output "$CREDS_FILE" \
    --encrypt "$TMPFILE"
# Overwrite before deleting.
dd if=/dev/urandom of="$TMPFILE" bs=1 count=64 2>/dev/null
rm -f "$TMPFILE"

chmod 600 "$CREDS_FILE"
echo "Encrypted credentials written to $CREDS_FILE"

# ---------------------------------------------------------------------------
# 4. Patch ~/.msmtprc
# ---------------------------------------------------------------------------

# Replace placeholder email and swap 'password' line for 'passwordeval'.
sed -i '' \
    -e "s|YOUR_GMAIL_ADDRESS|${GMAIL}|g" \
    -e "s|^    password.*|    passwordeval gpg --quiet --batch --no-tty --decrypt ${CREDS_FILE}|" \
    "$MSMTPRC"

echo "Updated $MSMTPRC"

# ---------------------------------------------------------------------------
# 5. Smoke test
# ---------------------------------------------------------------------------

echo ""
echo "Verifying decryption works..."
DECRYPTED="$(gpg --quiet --batch --no-tty --decrypt "$CREDS_FILE" 2>/dev/null)"
if [[ -z "$DECRYPTED" ]]; then
    echo "Warning: decryption returned empty output — check your GPG setup." >&2
else
    echo "Decryption OK."
fi

echo ""
echo "Setup complete. Test with:"
echo "  echo 'test' | msmtp --debug archerships@protonmail.ch"
