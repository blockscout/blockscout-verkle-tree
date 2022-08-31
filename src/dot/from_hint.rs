use verkle_trie::proof::UpdateHint;
use block_verkle_proof_extractor::{keyvals::KeyVals};
use ark_serialize::{CanonicalSerialize};
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;

type Node = usize;
type Edge<'a> = &'a (usize, usize, u8);
struct Graph {
    nodes: Vec<(String, Option<String>)>,
    edges: Vec<(usize,usize,u8)>
}

pub fn to_dot(uh: &UpdateHint, keyvals: &KeyVals, file_path: &PathBuf) -> Result<(), anyhow::Error> {
    use std::fs::File;
    let mut f = File::create(file_path)?;

    render_to(&mut f, uh, keyvals)
}

fn common_prefix(v1: &[u8], v2: &[u8]) -> usize {
    v1.iter().zip(v2).filter(|&(x, y)| x == y).count()
}

// We are transforming sorted Vec<paths, commitments> to the prefix-tree (dot)
pub fn render_to<W: Write>(
                        output: &mut W, data: &UpdateHint,
                        keyvals: &KeyVals)-> Result<(), anyhow::Error> {
    let mut nodes = vec![];
    let mut previous_items = Vec::<(Vec::<u8>, String)>::new();
    let mut edges = vec![];

    for (my_index, (path, comm)) in data.commitments_by_path.iter().enumerate() {
        let mut v_tmp = vec![];
        let _res = CanonicalSerialize::serialize(comm, &mut v_tmp);
        let commitment_in_hex = format!("0x{}", hex::encode(v_tmp.clone()));
        // firstly, we writing a common commitments (branch nodes)
        nodes.push((commitment_in_hex.clone(), None));

        // getting path info for edges
        for (index, item) in previous_items.iter().enumerate() {
            let len_current = item.0.len();

            if path.len() == len_current + 1 {
                let common = common_prefix(path, &item.0);

                if common == len_current {
                    edges.push((index, my_index, path[path.len() - 1]));
                }
            }
        }
        previous_items.push((path.clone(), commitment_in_hex));
    }

    // get extention nodes
    for (comm, val2) in data.depths_and_ext_by_stem.iter() {
        match val2.0 {
            // we are not intrested in POST-state keyvals
            verkle_trie::proof::ExtPresent::None => continue,
            // we don't use this nodes too
            verkle_trie::proof::ExtPresent::DifferentStem => continue,
            // ok
            verkle_trie::proof::ExtPresent::Present => {}
        };

        for item in previous_items.iter() {
            if item.0.len() == val2.1 as usize {
                let common = common_prefix(comm, &item.0);

                if common == val2.1 as usize {
                    // find element in nodes and update it
                    let index_element = nodes
                        .iter()
                        .position(|x| x.0 == item.1);
                    
                    match index_element {
                        Some(val) => nodes[val].1 = Some(hex::encode(comm)),
                        // we can't get here
                        None => {
                            tracing::error!("How could you get here?");
                            continue;
                        }
                    };

                    let prefix = comm.clone().to_vec();

                    // we need extention and len must be len 31
                    // iterate throwght keys and get them
                    for (indx, key) in keyvals.keys.iter().enumerate() {
                        if keyvals.values[indx].is_none() {
                            continue;
                        }

                        let common = common_prefix(&prefix, key);

                        if common == 31 {
                            let mut value = hex::encode(keyvals.values[indx].unwrap());
                            if value == "0000000000000000000000000000000000000000000000000000000000000000" {
                                value = "00..".to_owned();
                            }

                            nodes.push((format!("0x{value}"), None));
                            edges.push((index_element.unwrap() + 1_usize, nodes.len() - 1, key[31]));
                        }
                    }

                }
            }
        }
    }

    let graph = Graph { nodes, edges };

    match dot::render(&graph, output) {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow::anyhow!("Error with render dot {err}"))
    }
}

impl<'a> dot::Labeller<'a, Node, Edge<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example").unwrap()
    }

    fn node_id(&'a self, n: &Node) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label<'b>(&'b self, n: &Node) -> dot::LabelText<'b> {
        let node = self.nodes[*n].clone();
        let comm = node.0;
        let ext = match node.1 {
            Some(val) => format!("\next: 0x{}",val),
            None => "".to_owned()
        };

        dot::LabelText::LabelStr(
            Cow::Owned(format!("{comm}{ext}")))
    }

    fn edge_label<'b>(&'b self, edge: &Edge) -> dot::LabelText<'b> {
        let symbol = match edge.2 {
            1 => "c_1".to_owned(),
            2 => "c_2".to_owned(),
            _ => format!("{:0x}", edge.2)
        };

        dot::LabelText::LabelStr(Cow::Owned(symbol))
    }
}

impl<'a> dot::GraphWalk<'a, Node, Edge<'a>> for Graph {
    fn nodes(&self) -> dot::Nodes<'a,Node> {
        (0..self.nodes.len()).collect()
    }

    fn edges(&'a self) -> dot::Edges<'a,Edge<'a>> {
        self.edges.iter().collect()
    }

    fn source(&self, e: &Edge) -> Node {
        e.0
    }

    fn target(&self, e: &Edge) -> Node {
        e.1
    }
}
