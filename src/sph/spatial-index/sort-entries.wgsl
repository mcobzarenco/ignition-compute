#import ignition::spatial_index::common::{hash_position, SpatialIndexEntry}

@group(0) @binding(0) var<storage, read_write> entries: array<SpatialIndexEntry>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let num_particles = arrayLength(&entries);
    for (var i = 0u; i < num_particles; i += 1u) {
      for (var j = i + 1u; j < num_particles; j += 1u) {
	if entries[i].cell_key > entries[j].cell_key {
	    var aux = entries[j].cell_key;
	    entries[j].cell_key = entries[i].cell_key;
	    entries[i].cell_key = aux;
	  }
      }
    }
}
