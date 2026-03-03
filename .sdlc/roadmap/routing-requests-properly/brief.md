our server -> MCP integration needs to be able to route traffic properly, we should be able to use open code for tasks that we want to use gemini models and pick the proper model, and claude code for tasks we want the sonnet and opus and that agentive workflow dynamically

this means we have to list all of the routes that call models and then be able to dynamically configure them

the configuration should be (1) skill, (2) agentive provider (claude code or open code)
(3) model (sonnet-4.6, opus-4.6, etc)