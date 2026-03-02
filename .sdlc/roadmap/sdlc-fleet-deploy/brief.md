## The Brief

I want to deploy sdlc in a way that it's running for multiple projects simultaneously. The constraint is that sdlc state lives in a folder ( in each project repo).

I have a k3s environment at ~/Workspace/orchard9/k3s-fleet.

### Deployment options I'm thinking about

1. **Gitea + Longhorn volume**: Set up a shared volume on Longhorn or something with Gitea that has a bunch of repos, then run something like `sdlc ui` in those repos.

2. **1 docker per project**: A Docker container that runs sdlc configured to a specific project folder.

3. **1 docker for many**: A single Docker container that can run many sdlcs at once.

4. **Refactor to sdlc-many-ui**: Refactor sdlc UI so it can do `sdlc-many-ui` and watch over many projects.

### The scaling question
How would this scale?
- 10 projects
- 100 projects  
- 1000 projects
- 10,000 projects
- 100,000 projects