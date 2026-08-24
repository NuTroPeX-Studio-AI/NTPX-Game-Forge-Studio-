# Codex Start Task — G000 → G001

## Objective

Turn this handoff repository into the first runnable foundation of NTPX GameForge Studio Lab.

## Deliverables

### 1. Monorepo foundation
Choose a minimal, maintainable monorepo layout consistent with the architecture.
Do not introduce every planned dependency yet.

### 2. Command Center
Create a runnable React + TypeScript application with:
- project list
- project creation shell
- command input
- agent activity panel
- task queue
- build status panel
- test status panel
- logs panel
- settings placeholder

The UI can use mocked data for G001, but its interfaces must be shaped for real runtime integration.

### 3. Core contracts
Define typed interfaces/schemas for:
- Project
- Task
- Agent
- AgentRun
- Build
- TestRun
- EngineCapability
- EngineAdapter
- GFIRProject (minimal initial form)

### 4. Local persistence
Implement a minimal local persistence layer suitable for the Command Center.
Prefer SQLite or a thin repository abstraction that can begin in-process.

### 5. Event model
Define an internal event contract for:
- project.created
- task.created
- task.started
- task.completed
- task.failed
- agent.started
- agent.completed
- build.started
- build.completed
- build.failed
- test.started
- test.completed
- test.failed

Do not add NATS until an external broker is genuinely needed.

### 6. Tests
Add:
- unit tests for core contracts/validation
- UI smoke test
- project creation test
- task lifecycle test

### 7. Developer experience
Provide:
- setup commands
- run commands
- test commands
- lint/typecheck commands
- `.env.example`
- deterministic local startup

### 8. Documentation
Create/update:
- architecture decision for the chosen monorepo/build tooling
- active execution plan
- current status

## Acceptance criteria

G001 is complete when a developer can:
1. install dependencies,
2. start the Command Center,
3. create a local project,
4. see it persist,
5. create a mocked task,
6. see task lifecycle events,
7. run tests successfully,
8. understand the next step toward G002/G003.

## Do not do yet

- no Unreal integration
- no Unity integration
- no Roblox integration
- no console SDK integration
- no production cloud deployment
- no autonomous production writes
- no fake “working” game generation

## After completion

Report:
- files changed
- architecture decisions
- commands run
- tests/results
- known limitations
- exact next recommended task
