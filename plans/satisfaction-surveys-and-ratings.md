# Plan: Satisfaction Surveys & Rating System

**Status:** Planned  
**Created:** 2026-04-01

---

## Overview

Add a satisfaction survey and rating system so that:
- **Seekers** rate their experience with a provider after work is ongoing or done
- **Providers** rate their experience with a seeker after work is ongoing or done
- Aggregate ratings are displayed on service cards, detail pages, and suggestions to help future choices
- Request state is tracked through a full work lifecycle

---

## State Machine

Two separate status dimensions on each service request:

### Request Lifecycle (`status` column — existing, unchanged)
```
pending ──> accepted ──> (work begins)
pending ──> declined
pending ──> cancelled (by seeker)
accepted ──> cancelled (by seeker)
```

### Work Lifecycle (`work_status` column — new, only active when status = 'accepted')
```
not_started ──> in_progress ──> ongoing ──> done
                                    │          │
                                    └──────────┴──> Reviews unlocked
```

### Authorization Matrix

| Action | Who can do it |
|--------|--------------|
| Accept/Decline request | Provider |
| Cancel request | Seeker |
| Advance work_status | Provider |
| Leave review (as seeker) | Seeker, when work_status is "ongoing" or "done" |
| Leave review (as provider) | Provider, when work_status is "ongoing" or "done" |

---

## Database Changes

### Migration V005: `V005__create_reviews_and_work_status.sql`

```sql
-- Add work_status to service_requests for execution phase tracking
ALTER TABLE service_requests ADD COLUMN work_status TEXT NOT NULL 
    CHECK (work_status IN ('not_started', 'in_progress', 'ongoing', 'done')) 
    DEFAULT 'not_started';

CREATE INDEX idx_requests_work_status ON service_requests(work_status);

-- Reviews table: both seeker and provider can leave one review per request
CREATE TABLE reviews (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL REFERENCES service_requests(id),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    reviewee_id TEXT NOT NULL REFERENCES users(id),
    reviewer_role TEXT NOT NULL CHECK (reviewer_role IN ('seeker', 'provider')),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(request_id, reviewer_role)
);

CREATE INDEX idx_reviews_request ON reviews(request_id);
CREATE INDEX idx_reviews_reviewee ON reviews(reviewee_id);
CREATE INDEX idx_reviews_reviewer ON reviews(reviewer_id);
```

Key constraints:
- `UNIQUE(request_id, reviewer_role)` — each party can only leave one review per request
- `reviewee_id` is denormalized for efficient aggregate queries
- `rating` is 1–5 stars

---

## Backend Changes

### New handler: `handlers/reviews.rs`

Three endpoints:

1. **POST `/api/requests/{id}/review`** — Submit a review
   - Auth required; extract reviewer from JWT
   - Determine reviewer_role by matching against seeker_id / provider_id
   - Validate: work_status must be "ongoing" or "done"
   - Validate: rating 1–5, comment optional (max 1000 chars)
   - Enforce UNIQUE(request_id, reviewer_role) — no duplicate reviews
   - Body: `{ "rating": 4, "comment": "Great experience" }`

2. **GET `/api/requests/{id}/reviews`** — Get reviews for a request
   - Auth required; only the seeker or provider of that request can view
   - Returns 0, 1, or 2 review objects

3. **GET `/api/users/{id}/ratings`** — Aggregate ratings for a user
   - Public endpoint (no auth needed)
   - Returns:
   ```json
   {
     "avg_rating": 4.3,
     "review_count": 12,
     "as_provider": { "avg": 4.5, "count": 8 },
     "as_seeker": { "avg": 3.9, "count": 4 }
   }
   ```

### New endpoint in `handlers/services.rs`

4. **PATCH `/api/requests/{id}/work-status`** — Advance work status
   - Auth required; only the provider can advance
   - Validate: status must be "accepted"
   - Enforce forward-only transitions: not_started → in_progress → ongoing → done
   - Body: `{ "work_status": "in_progress" }`

### Modified queries in `handlers/services.rs`

- **`my_requests`**: Add `work_status`, both party names, and whether the current user has already reviewed (via LEFT JOIN on reviews)
- **`list` and `get`**: Add provider's average rating and review count via subquery

### Modified query in `handlers/surveys.rs`

- **`suggestions`**: Add bonus score points for highly-rated providers:
  - avg_rating >= 4.0: +2 points, reason: "Highly rated provider"
  - avg_rating >= 3.0: +1 point

### Route registration in `main.rs`

```
.route("/api/requests/{id}/review", web::post().to(handlers::reviews::submit))
.route("/api/requests/{id}/reviews", web::get().to(handlers::reviews::for_request))
.route("/api/users/{id}/ratings", web::get().to(handlers::reviews::user_ratings))
.route("/api/requests/{id}/work-status", web::patch().to(handlers::services::update_work_status))
```

---

## Frontend Changes

### API client (`api.js`)

Add methods:
```js
submitReview: (requestId, data) => request('POST', `/requests/${requestId}/review`, data),
getRequestReviews: (requestId) => request('GET', `/requests/${requestId}/reviews`),
getUserRatings: (userId) => request('GET', `/users/${userId}/ratings`),
updateWorkStatus: (requestId, workStatus) => request('PATCH', `/requests/${requestId}/work-status`, { work_status: workStatus }),
```

### Star rating component (`components/star-rating.js`)

Reusable functions:
- `renderStarRating(rating, max=5)` — read-only display (for service cards)
- `renderStarInput(name, currentValue)` — interactive clickable stars (for review form)
- Use Unicode stars (★/☆) styled with CSS

### Dashboard enhancements (`pages/shared/dashboard.js`)

The most significant UI change:

1. **Request table additions**:
   - "Progress" column showing work_status (color-coded badges)
   - "With" column showing the other party's name
   - "Actions" column with contextual buttons:
     - Provider: "Start Work" → "Mark In Progress" → "Mark Ongoing" → "Mark Done"
     - Both: "Leave Review" button (when work_status is ongoing/done, hidden if already reviewed)
     - Show "Reviewed ✓" badge when already submitted

2. **Inline review form** triggered by "Leave Review" button:
   - Star rating selector (1–5 clickable stars)
   - Comment textarea
   - Submit button → success message → disable form
   - Different prompt text based on reviewer_role:
     - Seeker prompt: "How was the service provided?"
     - Provider prompt: "How was your experience with this client?"

### Service cards with ratings (`browse.js`, `service-detail.js`, `suggestions.js`)

- Display provider's average rating and review count on service cards
- Star display: "★★★★☆ (12 reviews)"
- Detail page: full rating breakdown (as provider / as seeker)
- Suggestions: include rating in display

---

## i18n Keys

Add to all three locale files:

```
workStatus.not_started, workStatus.in_progress, workStatus.ongoing, workStatus.done

dashboard.colWorkStatus, dashboard.colActions, dashboard.colParty
dashboard.btnStart, dashboard.btnInProgress, dashboard.btnOngoing, dashboard.btnDone
dashboard.btnReview, dashboard.reviewSubmitted, dashboard.alreadyReviewed

review.title, review.ratingLabel, review.commentLabel, review.commentPlaceholder
review.submit, review.success, review.failed, review.alreadySubmitted
review.promptSeeker (How was the service provided?)
review.promptProvider (How was your experience with this client?)

rating.stars, rating.count, rating.noReviews, rating.asProvider, rating.asSeeker
rating.highlyRated

error.invalidRating, error.reviewNotAllowed, error.alreadyReviewed
```

---

## Implementation Phases

### Phase 1: Database & Backend Core
1. Create migration `V005__create_reviews_and_work_status.sql`
2. Create `models/review.rs`
3. Create `handlers/reviews.rs` (submit, for_request, user_ratings)
4. Add `update_work_status` handler to `handlers/services.rs`
5. Modify `my_requests` query to include work_status, party names, review status
6. Register routes in `main.rs`
7. Verify with cargo check

### Phase 2: Frontend Dashboard Enhancement
1. Add API methods to `api.js`
2. Create star-rating component
3. Enhance dashboard: work_status display, action buttons, review form
4. Add i18n keys to `en.json`
5. Add CSS for stars, badges, review form

### Phase 3: Rating Display on Browse/Detail/Suggestions
1. Modify backend `list` and `get` queries to include provider ratings
2. Update `browse.js` service cards with star ratings
3. Update `service-detail.js` with provider rating breakdown
4. Update `suggestions.js` to show and score by ratings
5. Enhance suggestions scoring in backend

### Phase 4: i18n & Polish
1. Add all keys to `es.json` and `pt.json`
2. Copy locale files to `public/i18n/locales/`
3. Test full flow: create request → accept → advance status → both leave reviews
4. Test in all three locales
5. Edge cases: double-submit prevention, network errors, cancelled requests

---

## CSS Additions

```css
/* Work status badges */
.work-badge { padding: 2px 8px; border-radius: 12px; font-size: 0.75rem; font-weight: 600; }
.work-badge.not-started { background: #e5e7eb; color: #374151; }
.work-badge.in-progress { background: #dbeafe; color: #1d4ed8; }
.work-badge.ongoing { background: #fef3c7; color: #92400e; }
.work-badge.done { background: #d1fae5; color: #065f46; }

/* Star rating display */
.stars { color: #f59e0b; letter-spacing: 1px; }
.stars-muted { color: #d1d5db; }
.review-count { font-size: 0.8rem; color: var(--color-text-muted); }

/* Interactive star input */
.star-input { cursor: pointer; font-size: 1.5rem; color: #d1d5db; transition: color 0.1s; }
.star-input.active, .star-input:hover { color: #f59e0b; }

/* Review form */
.review-form { border-top: 1px solid var(--color-border); padding-top: var(--space-md); margin-top: var(--space-md); }
```

---

## Considerations

- **SQLite ALTER TABLE with CHECK**: Works in SQLite 3.37.0+. The rusqlite `bundled` feature compiles a recent SQLite, so this should be fine.
- **Bidirectional reviews**: UI must clearly communicate who is being rated. Prompt text differs by role.
- **Dashboard complexity**: Extract request row rendering and action handling into helper functions to keep `dashboard.js` manageable.
- **Rating aggregation**: Computed on-the-fly via `AVG()`. Performant for this scale with the index on `reviewee_id`. Can materialize later if needed.
- **Delete existing `data/app.db`**: New migration adds a column, which requires a fresh migration run or an existing DB at migration V004.
