use gfa::{gfa::Line, parser::GFAParser};

use handlegraph::{
    handle::{Direction, Edge, Handle, NodeId},
    handlegraph::*,
    mutablehandlegraph::*,
    packed::*,
    pathhandlegraph::*,
};

use handlegraph::packedgraph::PackedGraph;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeRow {
    node_id: u64,
    seq: Vec<u8>,
    left_edges: Vec<u64>,
    right_edges: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathRow {
    path_name: String,
    handles: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OccurrenceRow {
    node_id: u64,
    path_name: String,
    step: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRecords {
    node_row_count: usize,
    path_row_count: usize,
    occur_row_count: usize,
    node_rows: Vec<NodeRow>,
    path_rows: Vec<PathRow>,
    occur_rows: Vec<OccurrenceRow>,
}

impl TestRecords {
    pub fn serialize(&self) -> Result<String> {
        use std::fmt::Write;

        let mut res = String::new();

        writeln!(
            res,
            "{}\t{}\t{}",
            self.node_row_count, self.path_row_count, self.occur_row_count
        )?;

        for node_row in self.node_rows.iter() {
            let seq_str = std::str::from_utf8(&node_row.seq)?;

            write!(res, "{}\t{}\t", node_row.node_id, seq_str)?;

            for (i, handle) in node_row.left_edges.iter().enumerate() {
                if i != 0 {
                    write!(res, ",")?;
                }
                write!(res, "{}", handle)?;
            }

            write!(res, "\t")?;

            for (i, handle) in node_row.right_edges.iter().enumerate() {
                if i != 0 {
                    write!(res, ",")?;
                }
                write!(res, "{}", handle)?;
            }

            writeln!(res)?;
        }

        for path_row in self.path_rows.iter() {
            write!(res, "{}\t", path_row.path_name)?;

            for (i, handle) in path_row.handles.iter().enumerate() {
                if i != 0 {
                    write!(res, ",")?;
                }
                write!(res, "{}", handle)?;
            }
            writeln!(res)?;
        }

        for occur_row in self.occur_rows.iter() {
            writeln!(
                res,
                "{}\t{}\t{}",
                occur_row.node_id, occur_row.path_name, occur_row.step
            )?;
        }

        Ok(res)
    }
}

pub fn get_node_rows(graph: &PackedGraph) -> Vec<NodeRow> {
    let mut rows: Vec<NodeRow> = graph
        .handles()
        .map(|handle| {
            let node_id = u64::from(handle.id());
            let seq = graph.sequence_vec(handle);

            let mut left_edges = graph
                .neighbors(handle, Direction::Left)
                .map(|h| h.0)
                .collect::<Vec<_>>();
            let mut right_edges = graph
                .neighbors(handle, Direction::Right)
                .map(|h| h.0)
                .collect::<Vec<_>>();

            left_edges.sort();
            right_edges.sort();

            NodeRow {
                node_id,
                seq,
                left_edges,
                right_edges,
            }
        })
        .collect();

    rows.sort();

    rows
}

pub fn get_path_rows(graph: &PackedGraph) -> Vec<PathRow> {
    let mut rows: Vec<PathRow> = graph
        .path_ids()
        .filter_map(|path_id| {
            let path_name_bytes = graph.get_path_name(path_id)?.collect::<Vec<_>>();
            let path_name_str = std::str::from_utf8(&path_name_bytes).ok()?;
            let path_name = String::from(path_name_str);

            let path_ref = graph.get_path_ref(path_id)?;
            let handles = path_ref
                .steps()
                .map(|step| step.handle().0)
                .collect::<Vec<_>>();

            Some(PathRow { path_name, handles })
        })
        .collect();

    rows.sort();

    rows
}

pub fn get_occur_rows(graph: &PackedGraph) -> Vec<OccurrenceRow> {
    let mut rows: Vec<OccurrenceRow> = graph
        .handles()
        .filter_map(|handle| {
            let occur_iter = graph.steps_on_handle(handle)?;
            let occurrences = occur_iter
                .filter_map(|(path_id, step)| {
                    let path_name_bytes = graph.get_path_name(path_id)?.collect::<Vec<_>>();
                    let path_name_str = std::str::from_utf8(&path_name_bytes).ok()?;
                    let path_name = String::from(path_name_str);

                    let step = step.pack();

                    let node_id = u64::from(handle.id());

                    Some(OccurrenceRow {
                        node_id,
                        path_name,
                        step,
                    })
                })
                .collect::<Vec<_>>();

            Some(occurrences)
        })
        .flatten()
        .collect();

    rows.sort();

    rows
}

pub fn get_graph_rows(graph: &PackedGraph) -> TestRecords {
    let node_rows = get_node_rows(graph);
    let path_rows = get_path_rows(graph);
    let occur_rows = get_occur_rows(graph);

    TestRecords {
        node_row_count: node_rows.len(),
        path_row_count: path_rows.len(),
        occur_row_count: occur_rows.len(),
        node_rows,
        path_rows,
        occur_rows,
    }
}
