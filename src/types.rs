use handlegraph::{handle::Direction, handlegraph::*, packed::*, pathhandlegraph::*};

use handlegraph::packedgraph::PackedGraph;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeRow {
    pub node_id: u64,
    pub seq: Vec<u8>,
    pub left_edges: Vec<u64>,
    pub right_edges: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathRow {
    pub path_name: String,
    pub handles: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OccurrenceRow {
    pub node_id: u64,
    pub path_name: String,
    pub step: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRecords {
    pub node_row_count: usize,
    pub path_row_count: usize,
    pub occur_row_count: usize,
    pub node_rows: Vec<NodeRow>,
    pub path_rows: Vec<PathRow>,
    pub occur_rows: Vec<OccurrenceRow>,
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

    pub fn deserialize(ser: &str) -> Option<Self> {
        let mut lines = ser.lines();

        let header = lines.next()?;
        let mut header = header.split("\t");

        let nodes = header.next()?;
        let node_row_count = nodes.parse::<usize>().ok()?;

        let paths = header.next()?;
        let path_row_count = paths.parse::<usize>().ok()?;

        let occurs = header.next()?;
        let occur_row_count = occurs.parse::<usize>().ok()?;

        let mut node_rows: Vec<NodeRow> = Vec::with_capacity(node_row_count);

        for _ in 0..node_row_count {
            let line = lines.next()?;
            let mut fields = line.split("\t");

            let node_id = fields.next().and_then(|f| f.parse::<u64>().ok())?;
            let seq = fields.next()?.bytes().collect::<Vec<_>>();

            let left_edges = fields.next()?;
            let left_edges = left_edges
                .split(",")
                .filter_map(|h| h.parse::<u64>().ok())
                .collect::<Vec<_>>();

            let right_edges = fields.next()?;
            let right_edges = right_edges
                .split(",")
                .filter_map(|h| h.parse::<u64>().ok())
                .collect::<Vec<_>>();

            node_rows.push(NodeRow {
                node_id,
                seq,
                left_edges,
                right_edges,
            });
        }

        let mut path_rows: Vec<PathRow> = Vec::with_capacity(path_row_count);

        for _ in 0..path_row_count {
            let line = lines.next()?;
            let mut fields = line.split("\t");

            let path_name = fields.next()?.to_string();

            let handles = fields.next()?;
            let handles = handles
                .split(",")
                .filter_map(|h| h.parse::<u64>().ok())
                .collect::<Vec<_>>();

            path_rows.push(PathRow { path_name, handles });
        }

        let mut occur_rows: Vec<OccurrenceRow> = Vec::with_capacity(occur_row_count);

        for _ in 0..occur_row_count {
            let line = lines.next()?;
            let mut fields = line.split("\t");

            let node_id = fields.next().and_then(|f| f.parse::<u64>().ok())?;
            let path_name = fields.next()?.to_string();
            let step = fields.next().and_then(|f| f.parse::<u64>().ok())?;

            occur_rows.push(OccurrenceRow {
                node_id,
                path_name,
                step,
            });
        }

        assert_eq!(node_row_count, node_rows.len());
        assert_eq!(path_row_count, path_rows.len());
        assert_eq!(occur_row_count, occur_rows.len());

        Some(TestRecords {
            node_row_count,
            path_row_count,
            occur_row_count,
            node_rows,
            path_rows,
            occur_rows,
        })
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
