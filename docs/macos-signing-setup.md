# macOS Signing and Notarization Setup

This document explains how to set up Apple Developer credentials for signing and notarizing SQLPage macOS binaries.

## Prerequisites

1. An Apple Developer account (https://developer.apple.com/)
2. Access to the SQLPage GitHub repository with secrets management permissions
3. Xcode command line tools installed on your local macOS machine

## Step 1: Create a Developer ID Application Certificate

1. Go to the [Apple Developer Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/certificates/list) page
2. Click the "+" button to add a new certificate
3. Select "Developer ID Application" under "Software"
4. Follow the instructions to create a Certificate Signing Request (CSR) from your Mac:
   ```bash
   openssl req -new -newkey rsa:2048 -nodes -keyout sqlpage.key -out sqlpage.csr
   ```
5. Upload the CSR file and complete the certificate creation
6. Download the certificate (.cer file)
7. Convert the certificate to P12 format for GitHub Actions:
   ```bash
   openssl x509 -in developerID_application.cer -inform DER -out sqlpage.pem -outform PEM
   openssl pkcs12 -export -out sqlpage.p12 -inkey sqlpage.key -in sqlpage.pem
   ```

## Step 2: Create an App Store Connect API Key for Notarization

1. Go to [App Store Connect](https://appstoreconnect.apple.com/access/api)
2. Click "Keys" and then "Generate API Key"
3. Set the following details:
   - Name: "SQLPage Notarization"
   - Access: "Developer"
4. Download the API key (.p8 file)

## Step 3: Add Secrets to GitHub Repository

Add the following secrets to the SQLPage GitHub repository:

1. `APPLE_SIGNING_CERTIFICATE_P12_BASE64`: Base64-encoded P12 certificate
   ```bash
   base64 -i sqlpage.p12 | pbcopy
   ```

2. `APPLE_SIGNING_CERTIFICATE_PASSWORD`: Password for the P12 certificate

3. `APPLE_SIGNING_IDENTITY`: The signing identity (e.g., "Developer ID Application: Your Name (TEAMID)")

4. `APPLE_NOTARIZATION_APPLE_ID`: Your Apple ID email address

5. `APPLE_NOTARIZATION_PASSWORD`: App-specific password for your Apple ID (create at https://appleid.apple.com/account/manage)

6. `APPLE_NOTARIZATION_TEAM_ID`: Your Apple Developer Team ID

## Step 4: Local Testing

To test the signing process locally:

1. Set the environment variables:
   ```bash
   export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
   export APPLE_NOTARIZATION_APPLE_ID="your@email.com"
   export APPLE_NOTARIZATION_PASSWORD="your-app-specific-password"
   export APPLE_NOTARIZATION_TEAM_ID="YOURTEAMID"
   ```

2. Run the signing script:
   ```bash
   ./scripts/sign-macos.sh
   ```

## Troubleshooting

- If you get "invalid signature" errors, make sure your certificate is valid and the signing identity matches
- If notarization fails, check the notarization logs with:
  ```bash
  xcrun notarytool log <submission-id> --apple-id your@email.com --password your-password --team-id YOURTEAMID
  ```
- Make sure your binary is properly signed before notarization
- Ensure your entitlements file is correct and matches the capabilities your app needs