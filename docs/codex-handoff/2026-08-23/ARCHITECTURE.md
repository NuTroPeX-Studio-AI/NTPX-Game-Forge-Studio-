# Architecture

## System overview

```text
USER
 |
 v
GAMEFORGE COMMAND CENTER
 |
 v
GENESIS DIRECTOR
 |
 +-- Planner
 +-- Task Scheduler
 +-- Agent Manager
 +-- Policy Engine
 +-- Project Memory
 |
 v
GAMEFORGE CORE
 |
 +-- GFIR
 +-- Project Graph
 +-- Asset Registry
 +-- Engine Router
 +-- Capability Registry
 |
 +----------------------+----------------------+
 v                      v                      v
AI WORKFORCE       ENGINE ADAPTERS        TOOL ADAPTERS
 |                      |                      |
Design                 Godot                  Blender
Engineering            Web                    Git
Art                    Unreal (later)         GitHub
Narrative              Unity (later)          Audio
QA                     Roblox (later)         Assets
 |
 v
BUILD + QA LAB
 |
 v
RELEASE FACTORY
```

## Core subsystems

### Genesis Director
Converts intent into requirements, plans, dependency graphs, tasks, agent assignments, review gates, and build decisions.

### GFIR
GameForge Intermediate Representation. Represents engine-independent game concepts such as:
- project
- world
- scene
- entity
- component
- character
- animation
- material
- shader
- physics
- camera
- input
- gameplay
- weapon
- vehicle
- inventory
- NPC
- dialogue
- quest
- audio
- UI
- networking
- save data
- platform
- build configuration

### Engine Router
Selects an engine based on capabilities, target platforms, project requirements, constraints, and policy.

### Adapter SDK
Defines the contract required for an engine adapter:
- capability discovery
- project creation
- import/export
- scene mutation
- code generation
- build invocation
- test invocation
- log capture
- artifact collection

### AI Workforce
Specialist roles:
- game designer
- systems designer
- gameplay engineer
- world builder
- character designer
- narrative agent
- asset agent
- QA agent
- build agent
- reviewer agent
- security agent

### Autonomous QA
Expected loop:
```text
Build -> Boot Test -> Playtest -> Detect Failure -> Diagnose -> Repair Proposal
      -> Rebuild -> Regression -> Report
```

### Security and provenance
- local/private default
- least privilege
- sandboxed tools
- secret isolation
- signed/hashed artifacts
- provenance metadata
- audit trail
- rollback
- dependency/SBOM scanning as the build matures

## First proof

A user request such as:

> Create a third-person prototype.

must become:

```text
Intent
 -> Requirements
 -> GFIR
 -> Task graph
 -> Agent work
 -> Engine adapter
 -> Real Godot project
 -> Player/camera/movement/level/UI
 -> Build
 -> Automated smoke test
 -> Repair on failure
 -> Playable artifact
```
