#define_import_path ignition::spatial_index::common

alias Position = vec2<f32>;

const U32_MAX: CellKey = 4294967295u;

struct SpatialIndexEntry {
    cell_key: u32,
    particle_index: u32,
}

struct NeighboursIter {
    cell_keys: array<u32, 9>,
    entry_index: u32,
    cell_key_index: u32,
}

fn new_neighbours_iter(position: Vec2, length_scale: f32) -> NeighboursIter {
    let offsets = neighbour_offsets(length_scale);
    var cell_keys = [
        get_cell_key(position + offsets[0]),
        get_cell_key(position + offsets[1]),
        get_cell_key(position + offsets[2]),
        get_cell_key(position + offsets[3]),
        get_cell_key(position + offsets[4]),
        get_cell_key(position + offsets[5]),
        get_cell_key(position + offsets[6]),
        get_cell_key(position + offsets[7]),
        get_cell_key(position + offsets[8]),
    ];

    for (var i = 0u; i < 9; i += 1u) {
      for (var j = i + 1u; j < 9; j += 1u) {
	if cell_keys[i] > cell_keys[j] {
	    var aux = cell_key[j];
	    cell_keys[j] = cell_keys[i];
	    cell_keys[i] = aux;
	}
      }
    }

    for (var i = 8u; i > 0; i -= 1u) {
      if cell_keys[i] == cell_keys[i - 1] {
	  cell_keys[i] = U32_MAX;
      }
    }

    NeighboursIter {
      
    }
}

fn neighbour_offsets(size: f32) -> array<vec2<f32>, 9> {
    return [
        vec2(-size, -size),
        vec2(-size, 0.0),
        vec2(-size, size),
        vec2(0.0, -size),
        vec2(0.0, 0.0),
        vec2(0.0, size),
        vec2(size, -size),
        vec2(size, 0.0),
        vec2(size, size),
    ];
}

fn get_potential_neighbours(iter: ptr<function, NeighboursIter>) {
}
  

fn get_cell_key(position: Position, length_scale: f32, num_particles: u32) -> u32 {
    return hash_position(position, params.length_scale) % num_particles;
}

fn hash_position(position: Position, length_scale: f32) -> u32 {
    let cell_id = floor(position / length_scale);
    var hash = 0u;
    hash = fxhash(hash, u32(cell_id.x));
    hash = fxhash(hash, u32(cell_id.y));
    return hash;
}

fn fxhash(hash: u32, value: u32) -> u32 {
    return (((hash << 5u) | (hash >> 27u)) ^ value) * FX_HASH_K;
}

const FX_HASH_K = 0x9e3779b9u;
