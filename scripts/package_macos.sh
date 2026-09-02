#!/usr/bin/env bash
# package_macos.sh — build, bundle, sign, notarize, and zip eDirStat for macOS.
#
# Local usage:
#   ./scripts/package_macos.sh                    # full pipeline (sign + notarize)
#   ./scripts/package_macos.sh --skip-notarize    # sign only, for quick iteration
#   ./scripts/package_macos.sh --ad-hoc           # ad-hoc signature (dev/CI without secrets)
#   ./scripts/package_macos.sh --target x86_64-apple-darwin
#
# CI usage: same — secrets come from the environment instead of the keychain.
# .github/workflows/ci.yml calls this with --ad-hoc on every commit;
# release.yml calls the full pipeline. There is ONE bundle assembly: here.
#
# Prereqs (one time):
#   xcode-select --install
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   xcrun notarytool store-credentials "notary" --apple-id ... --team-id ... --password ...

set -euo pipefail

# ---------- Config (override via environment) ----------
APP_NAME="eDirStat"
BINARY_NAME="edirstat"
BUNDLE_ID="com.edirstat.app"
ICON_SOURCE="assets/img/icon_256x.png"
MIN_MACOS="11.0"
TARGET="aarch64-apple-darwin"
NOTARY_PROFILE="${NOTARY_PROFILE:-notary}"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"   # auto-detected below if empty
SKIP_NOTARIZE=0
AD_HOC=0

# ---------- Args ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)        TARGET="$2"; shift 2 ;;
    --skip-notarize) SKIP_NOTARIZE=1; shift ;;
    --ad-hoc)        AD_HOC=1; SKIP_NOTARIZE=1; shift ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done

# ---------- Locate signing identity ----------
if [[ "$AD_HOC" -eq 0 ]]; then
  if [[ -z "$CODESIGN_IDENTITY" ]]; then
    CODESIGN_IDENTITY=$(security find-identity -v -p codesigning \
      | grep "Developer ID Application" | head -1 \
      | sed -E 's/.*"(.*)"/\1/') || true
  fi
  if [[ -z "$CODESIGN_IDENTITY" ]]; then
    echo "ERROR: no 'Developer ID Application' identity found in keychain." >&2
    echo "Create it in Xcode → Settings → Accounts → Manage Certificates," >&2
    echo "or pass --ad-hoc for an unsigned-adhoc development bundle." >&2
    exit 1
  fi
  echo "==> Signing identity: $CODESIGN_IDENTITY"
else
  echo "==> Signing identity: ad-hoc (-)"
fi
echo "==> Target:           $TARGET"

# ---------- Build ----------
echo "==> cargo build --release --target $TARGET"
cargo build --release --target "$TARGET"

VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
echo "==> Version: $VERSION"

# ---------- Assemble .app bundle ----------
APP_DIR="staging/$APP_NAME.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "target/$TARGET/release/$BINARY_NAME" "$APP_DIR/Contents/MacOS/$BINARY_NAME"

echo "==> Compiling icon"
rm -rf "$APP_NAME.iconset"
mkdir -p "$APP_NAME.iconset"
sips -z 16 16   "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_16x16.png"     >/dev/null
sips -z 32 32   "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_16x16@2x.png"  >/dev/null
sips -z 32 32   "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_32x32.png"     >/dev/null
sips -z 64 64   "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_32x32@2x.png"  >/dev/null
sips -z 128 128 "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_128x128.png"   >/dev/null
sips -z 256 256 "$ICON_SOURCE" --out "$APP_NAME.iconset/icon_128x128@2x.png">/dev/null
cp "$ICON_SOURCE" "$APP_NAME.iconset/icon_256x256.png"
iconutil -c icns "$APP_NAME.iconset" --output "$APP_DIR/Contents/Resources/icon.icns"
rm -rf "$APP_NAME.iconset"

cat > "$APP_DIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>          <string>$BINARY_NAME</string>
    <key>CFBundleIconFile</key>            <string>icon.icns</string>
    <key>CFBundleIdentifier</key>          <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>                <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>         <string>APPL</string>
    <key>CFBundleShortVersionString</key>  <string>$VERSION</string>
    <key>CFBundleVersion</key>             <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>      <string>$MIN_MACOS</string>
</dict>
</plist>
EOF

# ---------- Sign (inside-out: executable, then bundle) ----------
if [[ "$AD_HOC" -eq 1 ]]; then
  # Ad-hoc identity: hardened runtime, no entitlements, and no --timestamp
  # (ad-hoc has no cert chain for the timestamp server to countersign).
  echo "==> codesign (ad-hoc)"
  codesign --force --options runtime --sign - "$APP_DIR/Contents/MacOS/$BINARY_NAME"
  codesign --force --options runtime --sign - "$APP_DIR"
else
  echo "==> codesign"
  codesign --force --options runtime --timestamp \
    --sign "$CODESIGN_IDENTITY" \
    "$APP_DIR/Contents/MacOS/$BINARY_NAME"
  codesign --force --options runtime --timestamp \
    --sign "$CODESIGN_IDENTITY" \
    "$APP_DIR"
fi

OUT_ZIP="${BINARY_NAME}-macos-${TARGET%%-*}-${VERSION}.zip"

if [[ "$AD_HOC" -eq 1 ]]; then
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  echo "==> Done (ad-hoc, for development/CI): $OUT_ZIP"
  exit 0
fi

if [[ "$SKIP_NOTARIZE" -eq 1 ]]; then
  echo "==> Skipping notarization (--skip-notarize)"
  ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"
  echo "==> Done (signed, NOT notarized): $OUT_ZIP"
  exit 0
fi

# ---------- Notarize ----------
echo "==> notarytool submit (profile: $NOTARY_PROFILE)"
rm -f submission.zip
ditto -c -k --keepParent "$APP_DIR" submission.zip
xcrun notarytool submit submission.zip \
  --keychain-profile "$NOTARY_PROFILE" --wait
rm -f submission.zip

# ---------- Staple + final package ----------
echo "==> stapling ticket"
xcrun stapler staple "$APP_DIR"

ditto -c -k --keepParent "$APP_DIR" "$OUT_ZIP"

# ---------- Verify ----------
echo "==> Gatekeeper check:"
spctl -a -vvv "$APP_DIR"

echo "==> Done: $OUT_ZIP"
