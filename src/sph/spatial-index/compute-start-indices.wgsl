#import ignition::spatial_index::common::SpatialIndexEntry

@group(0) @binding(0) var<storage, read>        entries: array<SpatialIndexEntry>;
@group(0) @binding(1) var<storage, write> start_indices: array<u32>;

alias CellKey = u32;
const U32_MAX: CellKey = 4294967295u;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let entry_index = invocation_id.x;
    let num_particles = arrayLength(&entries);
    if (entry_index >= num_particles) {
        return;
    }

    let previous_cell_key = get_previous_cell_key(entry_index);
    let current_cell_key = entries[entry_index].cell_key;

    if (previous_cell_key != current_cell_key) {
        start_indices[current_cell_key] = entry_index;
    } else {
        start_indices[current_cell_key] = U32_MAX; 
    }
}

fn get_previous_cell_key(entry_index: u32) -> CellKey {
  if (entry_index == 0u) {
      return U32_MAX;
  } else {
      return entries[entry_index - 1u].cell_key;
  }
}
