# QA Plan: Iterate Button on FeatureDetail

## Unit Tests

### nextIterationSlug utility
1. `foo` with no existing slugs → `foo-v2`
2. `foo` with `foo-v2` existing → `foo-v3`
3. `foo-v3` with `foo-v3` existing → `foo-v4`
4. `foo-v2` with `foo-v2`, `foo-v3` existing → `foo-v4`
5. Slug with no `-vN` pattern and no collisions → appends `-v2`
6. Empty existing slugs array → `baseSlug-v2`

### FeatureDetail Iterate button
7. Button is visible only when `classification.action === 'done'`
8. Button shows loading state during ponder creation
9. Successful creation navigates to `/ponder/{newSlug}`

## Manual Verification

10. Navigate to a released feature → confirm "Iterate" button is visible in the green banner
11. Click "Iterate" → confirm ponder is created and page navigates to the new ponder
12. Verify the ponder has the correct title and brief

## Build Verification

13. `npm run build` completes without errors
14. TypeScript compilation passes
