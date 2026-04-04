# Plan: Google OAuth Login

**Status:** In Progress  
**Created:** 2026-04-04

---

## Prerequisites (Manual Setup)

1. Go to [Google Cloud Console](https://console.cloud.google.com/) → APIs & Services → Credentials
2. Create OAuth 2.0 Client ID (Web application)
3. Add `http://localhost:5173` to Authorized JavaScript origins
4. Copy the Client ID
5. Set `FS_GOOGLE_CLIENT_ID=<your-client-id>` in `backend/.env`

---

## Implementation Phases

### Phase 1: Database migration — make password_hash nullable, add google_id
### Phase 2: Backend — reqwest dep, google_login endpoint, update login for nullable password
### Phase 3: Frontend — GSI library, Google button, simplify register/forgot pages
### Phase 4: i18n + cleanup
