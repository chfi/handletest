use handletest::types::*;

use std::io::Read;

use gfa::{gfa::GFA, parser::GFAParser};

use handlegraph::{
    handle::{Edge, Handle},
    mutablehandlegraph::*,
    pathhandlegraph::*,
};

use handlegraph::packedgraph::PackedGraph;

use std::env;
use std::fs::File;
use std::process::exit;

enum Mode {
    Serialize,
    Deserialize,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let file_name = if let Some(name) = args.get(1) {
        name
    } else {
        eprintln!("provide a file name");
        exit(1);
    };

    let mode = if let Some(mode) = args.get(2) {
        match mode.as_str() {
            "write" => Mode::Serialize,
            "read" => Mode::Deserialize,
            _ => {
                eprintln!("provide a mode, `write` or `read`");
                exit(1);
            }
        }
    } else {
        eprintln!("provide a mode, `write` or `read`");
        exit(1);
    };

    match mode {
        Mode::Serialize => {
            let parser = GFAParser::new();
            let gfa: GFA<usize, ()> = parser.parse_file(file_name).unwrap();

            let min_id = gfa.segments.iter().map(|seg| seg.name).min().unwrap();
            let id_offset = if min_id == 0 { 1 } else { 0 };

            let mut graph = PackedGraph::default();

            for segment in gfa.segments.iter() {
                let id = (segment.name + id_offset) as u64;
                graph.create_handle(&segment.sequence, id);
            }

            for link in gfa.links.iter() {
                let from_id = (link.from_segment + id_offset) as u64;
                let to_id = (link.to_segment + id_offset) as u64;

                let from = Handle::new(from_id, link.from_orient);
                let to = Handle::new(to_id, link.to_orient);

                graph.create_edge(Edge(from, to));
            }

            for path in gfa.paths.iter() {
                let path_id = graph.create_path(&path.path_name, false).unwrap();
                for (node, orient) in path.iter() {
                    let handle = Handle::new(node as u64, orient);
                    graph.path_append_step(path_id, handle);
                }
            }

            let test_records = get_graph_rows(&graph);
            let test_ser = test_records.serialize().unwrap();
            println!("{}", test_ser);
        }
        Mode::Deserialize => {
            let mut file = File::open(file_name).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let test_records = TestRecords::deserialize(&contents).unwrap();

            println!("nodes  {}", test_records.node_row_count);
            println!("paths  {}", test_records.path_row_count);
            println!("occurs {}", test_records.occur_row_count);
        }
    }
}
