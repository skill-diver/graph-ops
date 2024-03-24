use std::collections::HashSet;

use pyo3::prelude::*;
use rand::Rng;

// Using CRC concepts
// Now we leave local mapping to the caller
#[pyfunction]
#[pyo3(
    text_signature = "(input_vertices:List[int], col_offsets:List[int],row:List[int],num_samples:int,replace:bool /)"
)]
pub fn neighbor_sample(
    input_vertices: Vec<i64>,
    col_offsets: Vec<usize>,
    row: Vec<i64>,
    num_samples: i64,
    replace: bool,
) -> (Vec<i64>, Vec<i64>, Vec<usize>) {
    if replace {
        neighbor_sample_generic::<true>(input_vertices, col_offsets, row, num_samples)
    } else {
        neighbor_sample_generic::<false>(input_vertices, col_offsets, row, num_samples)
    }
}

pub fn neighbor_sample_generic<const REPLACE: bool>(
    input_vertices: Vec<i64>,
    col_offsets: Vec<usize>,
    row: Vec<i64>,
    num_samples: i64,
) -> (Vec<i64>, Vec<i64>, Vec<usize>) {
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut edges = Vec::new();
    let mut sampled_nodes = HashSet::new();

    for (index, seed) in input_vertices.into_iter().enumerate() {
        sampled_nodes.insert(seed);
        let col_start = col_offsets[index];
        let col_end = col_offsets[index + 1];
        let col_count = col_end - col_start;

        if col_count == 0 {
            continue;
        }

        // if no sampling or sampling without replacement more neighbors than actual degree
        if num_samples < 0 || (!REPLACE && (num_samples as usize >= col_count)) {
            for (offset, nbr) in row.iter().enumerate().take(col_end).skip(col_start) {
                sampled_nodes.insert(*nbr);
                cols.push(seed);
                rows.push(*nbr);
                edges.push(offset);
            }
        } else if REPLACE {
            // random sampling with replacement
            let mut rng = rand::thread_rng();
            for _ in 0..num_samples {
                let offset = col_start + rng.gen_range(0..col_count);
                let nbr = row[offset];
                sampled_nodes.insert(nbr);
                cols.push(seed);
                rows.push(nbr);
                edges.push(offset);
            }
        } else {
            // random sampling without replacement
            let mut random_indices = HashSet::new();
            let mut rng = rand::thread_rng();
            for range_last in (col_count - num_samples as usize)..col_count {
                let mut rnd = rng.gen_range(0..(range_last + 1));
                // if already sampled, choose range_last
                if !random_indices.insert(rnd) {
                    rnd = range_last;
                    random_indices.insert(rnd);
                }
                let offset = col_start + rnd;
                let nbr = row[offset];
                sampled_nodes.insert(nbr);
                cols.push(seed);
                rows.push(nbr);
                edges.push(offset);
            }
        }
    }
    (rows, cols, edges)
}
