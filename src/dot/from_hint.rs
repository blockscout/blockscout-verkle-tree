use verkle_trie::proof::UpdateHint;
use ark_serialize::{CanonicalSerialize};
use std::borrow::Cow;
use std::io::Write;

type Node = usize;
type Edge<'a> = &'a (usize, usize, u8);
struct Graph {
    nodes: Vec<String>,
    edges: Vec<(usize,usize,u8)>
}

pub fn to_dot(uh: &UpdateHint, filename: &str) -> Result<(), anyhow::Error> {
    use std::fs::File;
    let mut f = File::create(filename)?;
    render_to(&mut f, uh)?;

    for (path, comm) in uh.commitments_by_path.iter() {
        println!("\tnode");
        let mut v = Vec::new();
        comm.serialize(&mut v)?;
        println!("comm :\t{:?}", hex::encode(v));
        print!("path :\t");
        for el in path {
            print!("{:x?} ", el);
        }
        println!();
    }
    println!("extra-info1:\n\n");

    for (val1, val2) in uh.depths_and_ext_by_stem.iter() {
        println!("val1 : {}", hex::encode(val1));
        println!("val2 : {:?}", val2);
    }
    Ok(())
}

// We are transforming sorted Vec<paths, commitments> to the prefix-tree (dot)
pub fn render_to<W: Write>(output: &mut W, data: &UpdateHint) -> Result<(), anyhow::Error> {
    let mut nodes = vec![];
    let mut previous_items = Vec::<(Vec::<u8>, String)>::new();
    let mut edges = vec![];

    for (my_index, (path, comm)) in data.commitments_by_path.iter().enumerate() {
        let mut v_tmp = vec![];
        let _res = CanonicalSerialize::serialize(comm, &mut v_tmp);
        let commitment_in_hex = format!("0x{}", hex::encode(v_tmp));
        nodes.push(commitment_in_hex.clone());

        for (index, item) in previous_items.iter().enumerate() {
            let len_current = item.0.len();

            if path.len() == len_current + 1 {
                let common = path.iter().zip(&item.0).filter(|&(x, y)| x == y).count();

                if common == len_current {
                    edges.push((index, my_index, path[path.len() - 1]));
                }
            }
        }
        previous_items.push((path.clone(), commitment_in_hex));
    }

    for (comm, val2) in data.depths_and_ext_by_stem.iter() {
        let commitment_in_hex = format!("0x{}", hex::encode(comm));

        // iter the prefix tree by commitment_in_hex to depth of vertices
        // and write to nodes
        println!("commitment : {}", commitment_in_hex);
        println!("ext and depth : {:?}", val2);
    }

    // go through all the keys and write them to the tree if there are exit nodes!
    let graph = Graph { nodes, edges };

    match dot::render(&graph, output) {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow::anyhow!("Error with render dot {err}"))
    }
}

impl<'a> dot::Labeller<'a, Node, Edge<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example2").unwrap()
    }

    fn node_id(&'a self, n: &Node) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label<'b>(&'b self, n: &Node) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes.get(*n).unwrap().into())
    }

    fn edge_label<'b>(&'b self, edge: &Edge) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(Cow::Owned(format!("{:0x}", edge.2)))
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
