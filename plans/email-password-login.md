# Plan: Email & Password Login

**Status:** Done  
**Created:** 2026-04-07

---

## Overview

Add traditional email/password registration and login to the frontend, alongside the existing Google OAuth. Users should be able to:
- **Register** with email, username, display name, and password
- **Log in** with email and password
- **Reset their password** via the forgot-password flow
- Continue using **Google Sign-In** as an alternative

The backend already supports all three endpoints (`register`, `login`, `reset-password`). This plan focuses on wiring up the frontend and making small backend improvements.

---

## Current State

| Layer | Email/Password | Google OAuth |
|-------|---------------|--------------|
| Backend endpoints | Implemented (`POST /api/auth/register`, `/login`, `/reset-password`) | Implemented (`POST /api/auth/google`) |
| Frontend UI | **Not wired up** — login & register pages only show Google button | Fully working |
| Password hashing | Argon2 (already in `auth.rs`) | N/A |

---

## Implementation Phases

### Phase 1: Registration Page — email/password form + Google button

**File:** `frontend/src/pages/shared/register.js`

1. Add a registration form with fields:
   - Email (required, validated)
   - Username (required)
   - Display name (required)
   - Password (required, min 8 chars)
   - Confirm password (must match)
2. Keep the Google Sign-In button below (or above) with a visual divider ("or")
3. On submit, call `POST /api/auth/register` via `api.js`
4. On success, call `setAuth(token, user)` and navigate to `/`
5. Show inline validation errors (email taken, username taken, password mismatch)
6. Add i18n keys for all new labels and error messages

### Phase 2: Login Page — email/password form + Google button

**File:** `frontend/src/pages/shared/login.js`

1. Add a login form with fields:
   - Email (required)
   - Password (required)
2. Keep the Google Sign-In button with a visual divider
3. Add a "Forgot password?" link pointing to `/forgot-password`
4. On submit, call `POST /api/auth/login` via `api.js`
5. On success, call `setAuth(token, user)` and navigate to `/`
6. Handle error cases:
   - Invalid credentials → generic "Invalid email or password" message
   - Google-only account → "This account uses Google Sign-In" message
7. Add i18n keys

### Phase 3: Forgot Password Page

**File:** `frontend/src/pages/shared/forgot-password.js`

1. Wire up the existing forgot-password page to `POST /api/auth/reset-password`
2. Form fields: email, username, new password, confirm new password
3. On success, show confirmation and redirect to `/login`
4. Handle errors (user not found, Google-only account)
5. Add i18n keys

### Phase 4: API Client — add auth helper functions

**File:** `frontend/src/api.js`

1. Add `register({ email, username, password, display_name })` function
2. Add `login({ email, password })` function
3. Add `resetPassword({ email, username, new_password })` function
4. All three call their respective backend endpoints and return the JSON response

### Phase 5: Backend Hardening

**File:** `backend/src/handlers/auth.rs`

1. Add input validation on `register`:
   - Email format validation
   - Password minimum length (8 chars)
   - Username length and character constraints
2. Return clear, i18n-friendly error codes (not just strings) for:
   - `email_taken`
   - `username_taken`
   - `invalid_credentials`
   - `google_only_account`
   - `user_not_found`
3. On `reset-password`, reject if account is Google-only (no password to reset)

### Phase 6: Styling & UX Polish

1. Style the email/password forms consistently with the existing design
2. Add a visual separator ("— or —") between the form and Google button
3. Add "Already have an account? Log in" link on register page
4. Add "Don't have an account? Register" link on login page
5. Ensure forms work well on mobile

### Phase 7: i18n

**Files:** `frontend/src/i18n/*.js`

Add translation keys for all supported languages:
- Form labels: email, username, display name, password, confirm password
- Buttons: register, log in, reset password
- Errors: email taken, username taken, password mismatch, invalid credentials, etc.
- Links: forgot password, already have account, don't have account
- Divider text: "or"

---

## Notes

- No database migration needed — `password_hash` is already nullable and Argon2 hashing is in place
- The backend `google_login` handler already links Google accounts to existing email/password users, so users who register with email first can later sign in with Google seamlessly
- JWT token format and expiration (24h) remain unchanged
- No email verification flow for now — can be added later as a separate feature
