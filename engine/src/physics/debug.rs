use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DebugStats {
    pub active_particles: u32,
    pub particles_in_bounds: u32,
    pub max_velocity_violations: u32,
    pub avg_velocity: f32,
    pub avg_position: [f32; 3],
    pub bounds_violations: [u32; 3], // x, y, z violations
    pub compute_time: Duration,
}

impl Default for DebugStats {
    fn default() -> Self {
        Self {
            active_particles: 0,
            particles_in_bounds: 0,
            max_velocity_violations: 0,
            avg_velocity: 0.0,
            avg_position: [0.0; 3],
            bounds_violations: [0; 3],
            compute_time: Duration::from_secs(0),
        }
    }
}

impl fmt::Display for DebugStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GPU Physics Debug Statistics:")?;
        writeln!(f, "Active Particles: {}", self.active_particles)?;
        writeln!(f, "Particles In Bounds: {}", self.particles_in_bounds)?;
        writeln!(
            f,
            "Max Velocity Violations: {}",
            self.max_velocity_violations
        )?;
        writeln!(f, "Average Velocity: {:.2}", self.avg_velocity)?;
        writeln!(
            f,
            "Average Position: [{:.2}, {:.2}, {:.2}]",
            self.avg_position[0], self.avg_position[1], self.avg_position[2]
        )?;
        writeln!(
            f,
            "Bounds Violations [x, y, z]: [{}, {}, {}]",
            self.bounds_violations[0], self.bounds_violations[1], self.bounds_violations[2]
        )?;
        writeln!(
            f,
            "Compute Time: {:.2}ms",
            self.compute_time.as_secs_f32() * 1000.0
        )
    }
}

#[derive(Debug)]
pub struct DebugVisualization {
    stats: DebugStats,
    enabled: bool,
    sample_rate: u32, // How often to update stats (in frames)
    frame_counter: u32,
}

impl DebugVisualization {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            stats: DebugStats::default(),
            enabled: false,
            sample_rate,
            frame_counter: 0,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn should_update(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        self.frame_counter += 1;
        if self.frame_counter >= self.sample_rate {
            self.frame_counter = 0;
            true
        } else {
            false
        }
    }

    pub fn update_stats(
        &mut self,
        particles: &[super::Particle],
        compute_time: Duration,
        bounds: [f32; 2],
        max_velocity: f32,
    ) {
        if !self.enabled {
            return;
        }

        let mut stats = DebugStats::default();
        stats.active_particles = particles.len() as u32;
        stats.compute_time = compute_time;

        let mut total_velocity = 0.0;
        let mut total_position = [0.0; 3];
        let mut in_bounds = 0;
        let mut velocity_violations = 0;
        let mut bounds_violations = [0; 3];

        for particle in particles {
            // Check velocity
            let velocity = [
                particle.velocity[0],
                particle.velocity[1],
                particle.velocity[2],
            ];
            let speed =
                (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2])
                    .sqrt();

            total_velocity += speed;
            if speed > max_velocity {
                velocity_violations += 1;
            }

            // Check position and bounds
            let mut particle_in_bounds = true;
            for i in 0..3 {
                let pos = particle.position[i];
                total_position[i] += pos;

                if pos < bounds[0] || pos > bounds[1] {
                    bounds_violations[i] += 1;
                    particle_in_bounds = false;
                }
            }

            if particle_in_bounds {
                in_bounds += 1;
            }
        }

        // Update averages
        if stats.active_particles > 0 {
            stats.avg_velocity = total_velocity / stats.active_particles as f32;
            for i in 0..3 {
                stats.avg_position[i] = total_position[i] / stats.active_particles as f32;
            }
        }

        stats.particles_in_bounds = in_bounds;
        stats.max_velocity_violations = velocity_violations;
        stats.bounds_violations = bounds_violations;

        self.stats = stats;
    }

    pub fn get_stats(&self) -> DebugStats {
        self.stats.clone()
    }

    pub fn get_stats_string(&self) -> String {
        format!("{}", self.stats)
    }
}

pub struct ParticleDebugView<'a> {
    particle: &'a super::Particle,
    pub index: usize,
}

impl<'a> fmt::Display for ParticleDebugView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Particle[{}] Pos:[{:.2}, {:.2}, {:.2}] Vel:[{:.2}, {:.2}, {:.2}]",
            self.index,
            self.particle.position[0],
            self.particle.position[1],
            self.particle.position[2],
            self.particle.velocity[0],
            self.particle.velocity[1],
            self.particle.velocity[2],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::Particle;
    use super::*;

    #[test]
    fn test_debug_visualization() {
        let mut debug = DebugVisualization::new(1);
        debug.enable();

        let test_particles = [
            Particle {
                position: [0.0, 0.0, 0.0, 1.0],
                velocity: [1.0, 1.0, 1.0, 0.0],
            },
            Particle {
                position: [2.0, 2.0, 2.0, 1.0],
                velocity: [2.0, 2.0, 2.0, 0.0],
            },
        ];

        debug.update_stats(&test_particles, Duration::from_millis(16), [-1.0, 1.0], 2.0);

        let stats = debug.get_stats();
        assert_eq!(stats.active_particles, 2);
        assert_eq!(stats.max_velocity_violations, 1); // Second particle exceeds max velocity of 2.0
        assert_eq!(stats.bounds_violations, [1, 1, 1]); // Second particle outside bounds
    }
}
