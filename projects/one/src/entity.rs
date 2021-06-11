use std::time::Duration;
use rayon::prelude::*;

/** Entity manager and simulator. */
pub struct Entities<T> {
	/** Collection of bundled entity class data. */
	bundles: Vec<ClassBundle<T>>
}
impl<T> Entities<T> {
	pub fn new() -> Self {
		Self {
			bundles: vec![]
		}
	}

	/** Register an entity class.
	 *
	 * An entity class consists of a group of zero or more entities and a
	 * procedure that operates on them individually or as a group. */
	pub fn register<F>(&mut self, procedure: F) -> Class
		where F: Fn(Duration, &mut [Entity<T>]) + Send + Sync + 'static {

		let bundle = ClassBundle {
			procedure: Box::new(procedure),
			entities: vec![]
		};

		let index = self.bundles.len();
		self.bundles.push(bundle);

		Class(index)
	}

	/** Spawns the given number of entities at the given class. The entities
	 * will be initialized with the given value, which requires that the
	 * entity data type be [Clone]. If that is not the case, use [spawn_with],
	 * instead.
	 *
	 * [Clone]: std::clone::Clone
	 */
	pub fn spawn(&mut self, class: Class, particles: usize, value: T)
		where T: Clone {
		let cont = &mut self.bundles[class.0].entities;

		let new_len = cont.len()
			.checked_add(particles)
			.expect("Adding this many particles would overflow usize!");
		cont.resize_with(new_len, || Entity { alive: true, payload: value.clone() });
	}

	/** Spawns the given number of entities at the given class. The provided
	 * function will be used to initialize each element, in order. */
	pub fn spawn_with<F>(&mut self, class: Class, particles: usize, mut f: F)
		where F: FnMut() -> T {
		let cont = &mut self.bundles[class.0].entities;

		let new_len = cont.len()
			.checked_add(particles)
			.expect("Adding this many particles would overflow usize!");
		cont.resize_with(new_len, || Entity { alive: true, payload: (f)() });
	}

	/** An iterator over all entity classes. */
	pub fn classes(&self) -> impl Iterator<Item = Class> {
		(0..self.bundles.len())
			.into_iter()
			.map(|index| Class(index))
	}

	/** The number of particles in all classes. */
	pub fn len(&self) -> usize {
		self.bundles.iter()
			.map(|bundle| bundle.entities.len())
			.sum()
	}

	/** Simulate all of the entity classes in this collection. */
	pub fn simulate(&mut self, delta: Duration)
		where T: Send {

		self.bundles
			.par_iter_mut()
			.for_each(move |bundle: &mut ClassBundle<T>| {
				/* Execute the procedure. */
				(bundle.procedure)(delta, &mut bundle.entities[..]);

				/* Clean up dead entities. */
				let mut first_dead = 0;
				let mut last_alive = bundle.entities.len().saturating_sub(1);

				loop {
					while let Some(true) = bundle.entities
						.get(first_dead)
						.map(|entity| entity.alive) {
						first_dead = first_dead.saturating_add(1);
					}
					while let Some(false) = bundle.entities
						.get(last_alive)
						.map(|entity| entity.alive) {
						last_alive = last_alive.saturating_sub(1);
					}
					if first_dead >= last_alive { break }

					bundle.entities.swap(first_dead, last_alive);
				}

				let alive = bundle.entities.iter()
					.take_while(|entity| entity.alive)
					.count();
				let _ = bundle.entities.drain(alive..);
			})
	}

	/** An iterator over the data in the entities in this collection. */
	pub fn entities(&self) -> impl Iterator<Item = &T> {
		self.bundles
			.iter()
			.flat_map(|bundle| bundle.entities.iter())
			.map(|entity| &entity.payload)
	}
}
impl<T> Default for Entities<T> {
	fn default() -> Self {
		Self::new()
	}
}

/** Bundle structure containing all the data associated with a particle class. */
struct ClassBundle<T> {
	/** The procedure to be applied to particles in this class. */
	procedure: Box<dyn Fn(Duration, &mut [Entity<T>]) + Send + Sync + 'static>,
	/** The vector containing all of the particles in this class. */
	entities: Vec<Entity<T>>,
}

pub struct Entity<T> {
	/** Whether the current particle is alive. */
	alive: bool,
	/** The data the users care about. */
	payload: T,
}
impl<T> Entity<T> {
	/** Marks this particle as being dead. */
	pub fn kill(&mut self) {
		self.alive = false;
	}

	/** Marks this particle as being alive. */
	pub fn revive(&mut self) {
		self.alive = true;
	}

	/** Whether this particle is alive. Dead particles  */
	pub fn alive(&self) -> bool {
		self.alive
	}
}
impl<T> AsRef<T> for Entity<T> {
	fn as_ref(&self) -> &T {
		&self.payload
	}
}
impl<T> AsMut<T> for Entity<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.payload
	}
}

/** A class of entities being operated on. */
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Class(usize);

