use support::{Camera, Projection};
use crate::entity::{Entities, Entity, Class};
use std::time::Duration;

/** Scene composition structure. Used by all of the major parts of the program
 * to change the scene and render it dynamically. */
pub struct Scene {
	pub camera: Camera,
	pub aspect: f32,

	pub light_position: [f32; 2],
	pub light_color: [f32; 3],

	pub snowflakes: Snowflakes,
}
impl Scene {
	pub fn new(aspect: f32) -> Self {
		Self {
			camera: Camera {
				projection: Projection::Orthographic {
					left: -1.0,
					right: 1.0,
					top: 1.0,
					bottom: -1.0,
					near: 1.0,
					far: 20.0,
				},
				position: [0.0, 0.0, 0.0,],
				yaw: 0.0,
				pitch: 0.0
			},
			aspect,
			light_position: [2.0, 2.0],
			light_color: [1.0, 1.0, 1.0],
			snowflakes: Snowflakes::new(),
		}
	}

	pub fn update(&mut self, delta: Duration) {
		self.snowflakes.entities.simulate(delta);

		self.snowflakes.spawn_timer += delta;
		while self.snowflakes.spawn_timer > Duration::from_millis(250) {
			let mut position = -1.2;
			self.snowflakes.entities.spawn_with(
				self.snowflakes.class,
				24,
				|| {
					position += 0.4;
					Snowflake {
						position: [position, 1.2],
						speed: [0.0, 0.0]
					}
				});

			self.snowflakes.spawn_timer -= Duration::from_millis(250);
		}
	}
}

/** Snowflake particle simulation bundle. */
pub struct Snowflakes {
	pub entities: Entities<Snowflake>,
	pub class: Class,
	pub spawn_timer: Duration
}
impl Snowflakes {
	/** Simulate snowflakes drifting in the wind. */
	pub fn simulate(delta: Duration, flakes: &mut [Entity<Snowflake>]) {
		for entity in flakes {
			let flake = entity.as_ref();

			/* Kill flakes which are already off-screen. */
			if flake.position[1] < -1.2 {
				entity.kill();
				continue
			}

			/* Make them drift. */
			let flake = entity.as_mut();

			flake.position[0] -= delta.as_secs_f32() * 0.5;
			flake.position[1] -= delta.as_secs_f32() * 0.4;
		}
	}

	pub fn new() -> Self {
		let mut entities = Entities::new();
		let class = entities.register(Self::simulate);

		Self { entities, class, spawn_timer: Default::default() }
	}


}

/** Structure holding the data for a single snowflake particle. */
pub struct Snowflake {
	pub position: [f32; 2],
	pub speed: [f32; 2],
}
