## Brief

Update the /sdlc-init command/template to immediately jump into creating the first milestone after bootstrapping vision/architecture/config.

The context came from observing this flow:

> /sdlc-ponder set up a proper foundation with full quality gate checks - it should be a hello world that has working core libraries for things like configs, logs, etc. It should integrate with our secrets properly, etc

The issue: after /sdlc-init bootstraps the project, the user is left with a blank SDLC state and has to manually run /sdlc-ponder or /sdlc-plan to start the first milestone. The init command should flow naturally into creating that first milestone immediately, treating the initial scope the user provided as the seed for Milestone 1.

Key question: what is the right handoff — does /sdlc-init itself commit to a milestone, or does it hand off to /sdlc-ponder-commit with the right context?