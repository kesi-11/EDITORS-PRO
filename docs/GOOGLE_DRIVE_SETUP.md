# Google Drive Cloud Sync Setup

This document describes how to configure Google Drive cloud sync for
EDITORS-PRO. The sync uses OAuth2 PKCE (Proof Key for Code Exchange)
for secure authentication without exposing any client secrets in the
app binary.

## Overview

The sync flow works as follows:

1. The user taps "Sign In with Google Drive" in the app.
2. The app opens a Chrome Custom Tab with the Google sign-in page.
3. The user signs in and grants the app permission to access its
   Drive files (only files created by the app — not the user's
   entire Drive).
4. Google redirects back to the app with an authorization code.
5. The app exchanges the code for access and refresh tokens.
6. The tokens are stored securely in the Android Keystore.
7. The app uses the access token to upload/download `.epp` project
   files to a dedicated `EDITORS-PRO` folder on Drive.

## Step 1: Create a Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/).
2. Click the project dropdown → **New Project**.
3. Name it "EDITORS-PRO" (or any name you like).
4. Click **Create**.

## Step 2: Enable the Google Drive API

1. In the Cloud Console, go to **APIs & Services → Library**.
2. Search for "Google Drive API".
3. Click **Enable**.

## Step 3: Configure the OAuth Consent Screen

1. Go to **APIs & Services → OAuth consent screen**.
2. Choose **External** (unless you have a Google Workspace account
   and want to keep it internal during development).
3. Fill in:
   - **App name**: EDITORS-PRO
   - **User support email**: your email
   - **Developer contact email**: your email
4. Click **Save and Continue**.
5. On the **Scopes** page, click **Add or Remove Scopes**:
   - Add `https://www.googleapis.com/auth/drive.file` (View and manage
     Google Drive files that you have opened or created with this app)
   - Add `openid` and `email` (for user identification)
6. Click **Save and Continue**.
7. On the **Test users** page, add your own Gmail address as a test
   user (required while the app is in "Testing" status).
8. Click **Save and Continue**.

> **Note**: While the app is in "Testing" status, only test users can
> authenticate. To allow anyone to sign in, you'll need to publish the
> app (requires Google's verification process — only do this when
> you're ready for public release).

## Step 4: Get your app's SHA-1 fingerprint

The OAuth2 client ID is tied to your app's signing key. You need the
SHA-1 fingerprint of your debug and/or release keystore.

### Debug keystore (for development)

```bash
keytool -list -v \
  -keystore ~/.android/debug.keystore \
  -alias androiddebugkey \
  -storepass android \
  -keypass android
```

Look for the `SHA1:` line.

### Release keystore (for production)

```bash
keytool -list -v \
  -keystore editors-pro-upload.jks \
  -alias upload
```

See `docs/RELEASE_SIGNING.md` for keystore setup.

## Step 5: Create the OAuth2 Client ID

1. Go to **APIs & Services → Credentials**.
2. Click **Create Credentials → OAuth client ID**.
3. Choose **Android** as the application type.
4. Fill in:
   - **Name**: EDITORS-PRO Android
   - **Package name**: `com.editorspro.editors_pro`
   - **SHA-1 certificate fingerprint**: paste the fingerprint from step 4
5. Click **Create**.
6. A dialog appears with your **Client ID**. Copy it — it looks like:
   ```
   123456789012-abcdefghijklmnopqrstuvwxyz.apps.googleusercontent.com
   ```

## Step 6: Configure the app

### 6a. Set the client ID in Dart

Open `lib/core/constants/cloud_config.dart` and set:

```dart
static const String googleDriveClientId =
    '123456789012-abcdefghijklmnopqrstuvwxyz.apps.googleusercontent.com';
```

### 6b. Set the redirect URI in AndroidManifest

Open `android/app/src/main/AndroidManifest.xml` and find the
`Phase E.20` intent filter. Replace `REPLACE_WITH_YOUR_CLIENT_ID` with
your client ID (the part before `.apps.googleusercontent.com`):

```xml
<data android:scheme="com.googleusercontent.apps.123456789012-abcdefghijklmnopqrstuvwxyz" />
```

> **Important**: The scheme must be `com.googleusercontent.apps.`
> followed by the **full** client ID (including the `.apps.googleusercontent.com`
> part). This is the standard reverse-DNS format that Google uses for
> Android OAuth2 redirect URIs.

## Step 7: Test the sync

1. Run the app: `flutter run`
2. Go to **Settings → Experimental → Cloud Sync** and enable it.
3. Go to **Settings → Cloud Sync → Cloud Provider** and select
   "Google Drive".
4. Tap **Sign In**.
5. A Chrome Custom Tab opens — sign in with the Google account you
   added as a test user in step 3.
6. Grant the requested permissions.
7. The app should show "Signed In" with your email address.
8. Open a project, then tap **Sync** to upload it to Drive.
9. Check your Drive — you should see an `EDITORS-PRO` folder with
   your project `.epp` file inside.

## Troubleshooting

### "redirect_uri_mismatch" error

The redirect URI in the app doesn't match what's registered in the
Cloud Console. Make sure:
- The client ID in `cloud_config.dart` matches the one from step 5.
- The scheme in `AndroidManifest.xml` is
  `com.googleusercontent.apps.<FULL_CLIENT_ID>`.
- The SHA-1 fingerprint matches your signing key.

### "access_denied" error

The user is not in the test users list. Add their Gmail address in
the Cloud Console under **OAuth consent screen → Test users**.

### "invalid_client" error

The client ID is wrong or the Android app's package name doesn't
match. Verify:
- Package name is `com.editorspro.editors_pro` (in
  `android/app/build.gradle.kts`).
- The client ID in `cloud_config.dart` is correct.

### Token refresh fails after app reinstall

When the app is reinstalled, the `flutter_secure_storage` keys are
wiped. The user needs to sign in again. This is expected behavior.

### "Google Drive client ID not configured"

You haven't set the client ID in `cloud_config.dart`. See step 6a.

## Security model

- **OAuth2 PKCE**: The app generates a random code verifier and sends
  a hash (code challenge) to Google with the auth request. When
  exchanging the code for tokens, the app proves it has the original
  verifier. This prevents interception attacks.
- **`drive.file` scope**: The app can only see files it created or
  that the user explicitly opened with the app. It cannot see the
  user's entire Drive.
- **Secure token storage**: Access and refresh tokens are stored in
  the Android Keystore via `flutter_secure_storage`. They're encrypted
  at rest and never appear in logs.
- **Token revocation**: On sign-out, the app revokes the access token
  on Google's side. The user can also revoke access at any time at
  https://myaccount.google.com/permissions.

## Privacy

- Only `.epp` project files (typically < 1 MB each) are synced.
- Source media (video files) are **never** uploaded — they stay local.
- The app does not read, modify, or share any other files on the
  user's Drive.
- Sync metadata (timestamps, file IDs) is stored locally in the
  app's database.

## Publishing

When you're ready to publish:

1. Go to **OAuth consent screen → Publish app**.
2. Submit for verification (Google reviews the app to ensure it
   complies with their API policies).
3. Verification typically takes 2-6 weeks for apps requesting
   sensitive scopes. The `drive.file` scope is considered
   "restricted" but the verification is usually straightforward.
4. Once verified, any Google user can sign in (not just test users).
