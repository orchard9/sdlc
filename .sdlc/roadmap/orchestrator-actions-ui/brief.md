The backend orchestrator is fully implemented (Action model, ActionDb in redb, tick daemon CLI, webhook ingestion + routing REST API). But the frontend has zero UI for it. Specifically missing:

- Frontend Actions page
- Setup sidebar → Actions nav item  
- API client methods for orchestrator

The orchestrator allows scheduling autonomous actions (scheduled ticks or webhook-triggered) that invoke tools. Users need to be able to: see what actions are scheduled, add new actions, manage webhook routes, and see action history/status. This should live under Setup in the nav sidebar alongside Tools, Secrets, Agents.