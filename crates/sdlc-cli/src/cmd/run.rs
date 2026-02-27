// `sdlc run` has been removed.
//
// sdlc is a pure state machine and directive producer. It does not dispatch
// to agent backends or spawn subprocesses. Use `sdlc next --for <slug> --json`
// to get the structured action directive and pass it to your directive consumer.
