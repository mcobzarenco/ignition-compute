use bevy::{core::Pod, prelude::*, render::render_resource::ShaderType, utils::RandomState};
use bytemuck::Zeroable;
use itertools::Either;
use rayon::prelude::ParallelSliceMut;
use std::num::NonZeroUsize;

#[derive(ShaderType, Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
struct DenormalizedNeighbourIndices {
    start: u32,
    end: u32,
}

type CellKey = u32;
type ParticleIndex = u32;

#[derive(ShaderType, Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
struct SpatialIndexEntry {
    cell_key: CellKey,
    particle_index: ParticleIndex,
}

#[derive(Debug, Resource)]
pub struct SpatialIndex {
    entries: Vec<SpatialIndexEntry>,
    start_indices: Vec<Option<NonZeroUsize>>,
    hasher: RandomState,
    num_particles: usize,
    cell_size: f32,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(num_particles: usize) -> Self {
        Self {
            entries: Vec::with_capacity(num_particles),
            start_indices: Vec::with_capacity(Self::hash_map_size(num_particles)),
            hasher: RandomState::with_seed(42),
            num_particles: 0,
            cell_size: 0.0,
        }
    }

    #[inline]
    fn hash_map_size(num_particles: usize) -> usize {
        2 * num_particles
    }

    #[inline]
    pub fn potential_neighbours<'a>(&'a self, position: Vec2) -> impl Iterator<Item = u32> + 'a {
        const NUM_QUERY_CELLS: usize = 9;

        if self.num_particles == 0 {
            Either::Left(std::iter::empty())
        } else {
            let mut query_cells: [CellKey; NUM_QUERY_CELLS] = self
                .offsets()
                .map(|offset| self.position_to_cell(offset + position));
            query_cells.sort_unstable();

            let neighbours = itertools::chain!(
                // 0
                self.entities_in_cell_key(query_cells[0]),
                // 1
                if query_cells[0] != query_cells[1] {
                    Either::Right(self.entities_in_cell_key(query_cells[1]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 2
                if query_cells[1] != query_cells[2] {
                    Either::Right(self.entities_in_cell_key(query_cells[2]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 3
                if query_cells[2] != query_cells[3] {
                    Either::Right(self.entities_in_cell_key(query_cells[3]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 4
                if query_cells[3] != query_cells[4] {
                    Either::Right(self.entities_in_cell_key(query_cells[4]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 5
                if query_cells[4] != query_cells[5] {
                    Either::Right(self.entities_in_cell_key(query_cells[5]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 6
                if query_cells[5] != query_cells[6] {
                    Either::Right(self.entities_in_cell_key(query_cells[6]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 7
                if query_cells[6] != query_cells[7] {
                    Either::Right(self.entities_in_cell_key(query_cells[7]))
                } else {
                    Either::Left(std::iter::empty())
                },
                // 8
                if query_cells[7] != query_cells[8] {
                    Either::Right(self.entities_in_cell_key(query_cells[8]))
                } else {
                    Either::Left(std::iter::empty())
                },
            );
            Either::Right(neighbours)
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn entities_in_cell<'a>(&'a self, position: Vec2) -> impl Iterator<Item = u32> + 'a {
        if self.num_particles == 0 {
            Either::Left(std::iter::empty())
        } else {
            Either::Right({
                let cell_key = self.position_to_cell(position);
                self.entities_in_cell_key(cell_key)
            })
        }
    }

    pub fn update(
        &mut self,
        num_particles: usize,
        cell_size: f32,
        particles: impl IntoIterator<Item = (u32, Vec2)>,
    ) {
        debug_assert!(cell_size > 0.0);

        // Update parameters
        self.num_particles = num_particles;
        self.cell_size = cell_size;

        // Clear and initialize `start_indices`
        self.start_indices.clear();
        self.start_indices
            .resize(Self::hash_map_size(num_particles), None);

        // Clear and collect `entries`
        self.entries.clear();
        for (entity, position) in particles {
            let cell_key = self.position_to_cell(position);
            self.entries.push(SpatialIndexEntry {
                cell_key,
                particle_index: entity,
            });
        }
        self.finish();

        debug_assert_eq!(num_particles, self.entries.len());
        debug_assert_eq!(Self::hash_map_size(num_particles), self.start_indices.len());
    }

    #[inline]
    fn entities_in_cell_key<'a>(&'a self, cell_key: CellKey) -> impl Iterator<Item = u32> + 'a {
        let maybe_start_index =
            self.start_indices[cell_key as usize].map(|nonzero| nonzero.get() - 1);

        let potential_matches = maybe_start_index
            .map(|start_index| &self.entries[start_index..])
            .unwrap_or(&[]);

        potential_matches
            .iter()
            .take_while(move |entry| entry.cell_key == cell_key)
            .map(|entry| entry.particle_index)
    }

    #[inline]
    fn position_to_cell(&self, position: Vec2) -> CellKey {
        let cell_indices = (position / self.cell_size).floor();
        u32::try_from(
            self.hasher
                .hash_one((cell_indices.x as i32, cell_indices.y as i32))
                % (Self::hash_map_size(self.num_particles) as u64),
        )
        .unwrap()
    }

    #[inline]
    fn finish(&mut self) {
        if self.num_particles == 0 {
            return;
        }

        // Sort the entries
        self.entries
            .par_sort_unstable_by_key(|entry| entry.cell_key);

        // Compute start_indices
        self.start_indices[self.entries[0].cell_key as usize] = Some(NonZeroUsize::new(1).unwrap());

        for (index, (previous_cell_key, current_cell_key)) in self.entries[..self.entries.len() - 1]
            .iter()
            .map(|entry| entry.cell_key)
            .zip(self.entries[1..].iter().map(|entry| entry.cell_key))
            .enumerate()
        {
            if previous_cell_key != current_cell_key {
                self.start_indices[current_cell_key as usize] =
                    Some(NonZeroUsize::new(index + 2).unwrap());
            }
        }
    }

    #[inline]
    fn offsets(&self) -> [Vec2; 9] {
        let size = self.cell_size;
        [
            Vec2::new(-size, -size),
            Vec2::new(-size, 0.0),
            Vec2::new(-size, size),
            Vec2::new(0.0, -size),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, size),
            Vec2::new(size, -size),
            Vec2::new(size, 0.0),
            Vec2::new(size, size),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;


    fn random_points(num_points: usize) -> Vec<Vec2> {
        let mut rng = rand::thread_rng();

        let mut points = Vec::new();
        for _ in 0..num_points {
            points.push(Vec2::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-3.0..3.0),
            ))
        }
        points
    }

    #[test]
    fn test_spatial_index_empty() {
        let mut index = SpatialIndex::new();
        index.update(0, 1.0, std::iter::empty());
        assert_eq!(index.potential_neighbours(Vec2::new(1.0, -0.1)).count(), 0);
    }

    #[test]
    fn test_spatial_index() {
        let radius = 1.0;
        let points = random_points(1000);

        let mut index = SpatialIndex::new();
        index.update(
            points.len(),
            radius,
            points
                .iter()
                .enumerate()
                .map(|(index, &point)| (u32::try_from(index).unwrap(), point)),
        );

        // println!("{}", index);
        for (i, &x_i) in points.iter().enumerate() {
            let mut index_neighbours = index
                .potential_neighbours(x_i)
                .map(|entity| usize::try_from(entity).unwrap())
                .filter_map(|j| ((x_i - points[j]).length() <= radius).then(|| (j, points[j])))
                .collect::<Vec<(usize, Vec2)>>();
            index_neighbours.sort_by_key(|x| x.0);

            let actual_neighbours = points
                .iter()
                .enumerate()
                .filter_map(|(j, &x_j)| ((x_i - x_j).length() <= radius).then(|| (j, x_j)))
                .collect::<Vec<(usize, Vec2)>>();

            assert_eq!(
                index_neighbours, actual_neighbours,
                "The neighbours of point {} {} are wrong",
                i, x_i
            );
        }
    }

    #[test]
    fn test_denormalized_neighbours() {
        let radius = 1.0;
        let points = random_points(10);

        let mut spatial_index = SpatialIndex::new();
        spatial_index.update(
            points.len(),
            radius,
            points
                .iter()
                .enumerate()
                .map(|(index, &point)| (u32::try_from(index).unwrap(), point)),
        );

        let mut neighbour_indices = Vec::new();
        let mut neighbour_indices_start = Vec::new();
        let mut current_index = 0;

        neighbour_indices.clear();
        for &point in points.iter() {
            neighbour_indices_start.push(current_index);
            for neighbour_index in spatial_index.potential_neighbours(point) {
                current_index += 1;
                neighbour_indices.push(neighbour_index); 
            }
        }
        neighbour_indices_start.push(neighbour_indices.len());

        println!("{:?}", neighbour_indices);
        println!("{:?}", neighbour_indices_start);
    }
}
