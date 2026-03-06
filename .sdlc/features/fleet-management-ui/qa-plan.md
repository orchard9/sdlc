# QA Plan: fleet-management-ui

## Q1: Running instances render correctly
- Load page with fleet data available
- Verify each instance card shows: name, URL, status dot with correct color, active milestone, feature count, agent badge when applicable
- Click a card and verify it opens the instance URL in a new tab

## Q2: Available repos render correctly
- Verify Available section shows repos without running instances
- Verify each card shows repo name and description (truncated if long)
- Verify first-time context line appears above the section

## Q3: Start provisioning flow
- Click Start on an available repo
- Verify button shows provisioning/spinner state and is disabled
- Verify POST /api/hub/provision is called with correct slug
- On SSE fleet_provisioned event, verify repo moves from Available to Running

## Q4: Import flow — success
- Enter a valid URL in the import field
- Click Import
- Verify POST /api/hub/import is called with URL and optional PAT
- Verify progress states: importing → provisioning → done
- Verify new instance appears in Running section

## Q5: Import flow — error
- Enter an invalid URL or trigger a server error
- Verify inline error message appears below the form
- Verify form remains editable for retry

## Q6: Search filtering
- Type a search term
- Verify both Running and Available sections filter simultaneously
- Verify filter count shows "N of M" format
- Clear search and verify all items reappear

## Q7: Search autofocus
- Load the page fresh
- Verify the search input has focus without clicking

## Q8: Agent summary bar
- Verify "N agents running across M projects" appears when agents are active
- Verify "No active agents" when none are running
- Verify SSE updates change the summary in real time

## Q9: SSE live updates
- Verify fleet_updated events update instance cards in place (status changes)
- Verify fleet_provisioned events add new instances and remove from available
- Verify fleet_agent_status events update the agent summary bar
- Verify no polling — only SSE drives updates

## Q10: Empty states
- With no instances and no available repos: verify combined empty state
- With instances but no available repos: verify "All repos have running instances" message
- With no instances but available repos: verify "No instances running" message

## Q11: Graceful degradation
- If GET /api/hub/fleet returns error: verify fallback to heartbeat data or empty state
- If GET /api/hub/available returns error: verify Available section is hidden
- If GET /api/hub/agents returns error: verify agent summary bar is hidden
- Verify each section loads independently

## Q12: Build verification
- Verify `cargo clippy --all -- -D warnings` passes
- Verify `SDLC_NO_NPM=1 cargo test --all` passes
- Verify `cd frontend && npm run build` succeeds without errors
