# Memory Allocation

This diagram illustrates the memory layout and highlights the use of memory pooling and cache-friendly data structures within the AshEngine physics system.

```mermaid
graph LR
    subgraph CPU Memory
        A[Physics World] --> B(Object Array)
        A --> C(Constraint Array)
        B --> B1[Rigid Body Data]
        B --> B2[Deformable Body Data]
        C --> C1[Constraint Data (Pooled)]
        D[Spatial Hash] --> D1(Grid Cells)
        D --> D2(Object Indices)
        E[Cache-Friendly Spatial Hash] --> E1(Morton Codes)
        E --> E2(Object Indices)
        style C1 fill:#ccf,stroke:#333,stroke-width:2px
    end
    subgraph GPU Memory
        F[Particle Data (PBD)] --> F1(Positions)
        F --> F2(Velocities)
        F --> F3(Masses)
    end
```

**Explanation:**

- **CPU Memory:**

  - **Physics World:** The central manager for the physics simulation.
  - **Object Array:** Stores data for both rigid bodies and deformable bodies.
  - **Constraint Array:** Stores constraint data. Constraint data is pooled (indicated by the highlighted box) to reduce allocation overhead.
  - **Spatial Hash:** Used for broad-phase collision detection.
    - **Grid Cells:** Stores the grid cells of the spatial hash.
    - **Object Indices:** Stores indices of objects within each cell.
  - **Cache-Friendly Spatial Hash:** An alternative spatial hash that uses Morton codes for improved cache efficiency.
    - **Morton Codes:** Stores Morton codes representing cell locations.
    - **Object Indices:** Stores indices of objects within each cell.

- **GPU Memory (for GPU-accelerated PBD):**
  - **Particle Data (PBD):** Stores particle data for deformable bodies.
    - **Positions:** Particle positions.
    - **Velocities:** Particle velocities.
    - **Masses:** Particle masses.

This diagram shows how memory is organized and highlights the use of memory pooling for constraints and the cache-friendly spatial hash for improved performance.
