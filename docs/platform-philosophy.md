# DUAN Platform Philosophy

DUAN has two design layers.

The runtime layer answers how one simulation world behaves internally. That philosophy lives with the runtime documentation: entities express intent, domains decide facts, snapshots stabilize visibility, and lifecycle side effects commit at explicit points.

The platform layer answers how simulation capability is authored, reused, assembled, edited, built, and delivered. This document defines that platform philosophy.

## Rust Is The Behavior Boundary

Simulation behavior belongs in Rust packages.

Components, entities, domains, events, reactions, observers, factories, schema shape, and editor metadata are authored from Rust source. Scenario manifests may select and configure those capabilities, but they do not become the place where algorithms live.

This keeps the highest-performance path ordinary Rust and prevents product tooling from inventing a second simulation language that has to mirror the runtime.

## Packages Are Capability Units

A DUAN package is a Cargo package that exposes simulation capability.

The Cargo package name is the package id. Package item ids are stable references inside that package and use `<package-id>/<id>`. Item kind is metadata, not an id path segment.

Packages are split by ownership and release boundary, not by mechanical type. A package can contain components, entities, domains, events, reactions, and observers when those items ship as one capability.

## Scenarios Assemble, They Do Not Program

A scenario manifest describes which package items are used and what initial values or parameters they receive.

It can configure entities, component values, domain parameters, reactions, run options, outputs, and experiment batches. It cannot define `Entity::tick`, `Domain::compute`, `Reaction::react`, or physics/search/planning algorithms.

When a scenario needs new behavior, the behavior is added to a Rust package first.

## Editors Configure Capabilities

The editor reads package schemas and edits scenario projects. It does not edit business algorithms.

This makes the editor useful for configuration, inspection, validation, experiment setup, package installation, and run result review without turning visual nodes into a second source of behavior truth.

## Generated Runners Keep The Hot Path Static

DUAN favors generated Rust runners that statically link the selected package set.

Changing component values or run parameters should not rebuild the runner. Changing the package set should regenerate and rebuild it. This keeps normal parameter work fast while preserving compiled Rust performance when capability changes.

## Caches Are Artifacts

Generated `duan.toml`, `duan-package.toml`, `schemas/**`, and editor metadata files are installation, validation, publishing, or delivery artifacts.

They exist so tools can inspect packages without parsing arbitrary source. They are not the normal authoring source of truth and should not force users to maintain duplicate item lists.

## The Repository Root Is A Product Entry

The repository root is a product and documentation collection. It does not need to be a Cargo package or a root Cargo workspace by default.

The user-facing Cargo crate can be a thin `duan` facade package under `packages/duan/`. The runtime implementation should be `duan-runtime`, not the only unsuffixed framework crate.

## Platform Must Not Pollute Runtime

Package ids, schema metadata, editor labels, install caches, and scenario project files are assembly-layer concepts.

They must not leak into `World::step`, component storage, scheduler hot paths, or runtime semantics. The runtime can expose small assembly hooks, but platform convenience must not turn the core into a string-driven dynamic ECS.
