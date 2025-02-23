# Current CPU Parallelism

This diagram illustrates the current CPU parallelization strategy within the AshEngine physics system. The main phases of the physics update (`sub_update` function in `physics.rs`) are shown, with highlights indicating the sections that utilize parallel processing via the `rayon` crate.

```mermaid
sequenceDiagram
    participant MainThread
    participant WorkerThread1
    participant WorkerThread2
    participant ...
    MainThread->>+PhysicsWorld: sub_update(dt)
    Note over MainThread, ...: Phase 1: Position Update (Parallel)
    MainThread->>WorkerThread1: Update RigidBody 1
    MainThread->>WorkerThread2: Update DeformableBody 1
    MainThread->>...: ...
    WorkerThread1-->>MainThread: Done
    WorkerThread2-->>MainThread: Done
    ...-->>MainThread: Done
    Note over MainThread, ...: Phase 2: Broad-phase (Parallel)
    MainThread->>+SpatialHash: Insert Objects (Parallel)
    SpatialHash-->>-MainThread: Potential Collisions
    Note over MainThread, ...: Phase 3: Narrow-phase (Parallel)
    MainThread->>WorkerThread1: Check Collision (Obj1, Obj2)
    MainThread->>WorkerThread2: Check Collision (Obj3, Obj4)
    MainThread->>...: ...
    WorkerThread1-->>MainThread: CollisionConstraint?
    WorkerThread2-->>MainThread: CollisionConstraint?
    ...-->>MainThread: CollisionConstraint?
    Note over MainThread, ...: Phase 4: Constraint Solving (Island-based Parallel)
    MainThread->>+IslandSolver: Solve Constraints
    IslandSolver-->>-MainThread:
    Note over MainThread, ...: Phase 5: Velocity Update (Parallel for Deformable Bodies)
    MainThread->>WorkerThread1: Update DeformableBody 1 Velocities
    MainThread->>...: ...
    WorkerThread1-->>MainThread: Done
    ...-->>MainThread: Done
    MainThread->>MainThread: Cleanup Collision Constraints
    PhysicsWorld-->>-MainThread:
```

**Explanation:**

1.  **Position Update:** Object positions and velocities are updated in parallel using `rayon`'s parallel iterators. Both rigid bodies and deformable bodies are handled.
2.  **Broad-phase Collision Detection:** Objects are inserted into the spatial hash in parallel, again using `rayon`.
3.  **Narrow-phase Collision Detection:** Potential collision pairs identified by the broad-phase are processed in parallel to determine actual collisions and generate collision constraints.
4.  **Constraint Solving:** The island-based constraint solver groups independent constraints into "islands" and solves them in parallel.
5.  **Velocity Update:** Velocities of deformable bodies are updated in parallel.
6.  **Cleanup:** Temporary collision constraints are removed.
