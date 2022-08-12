use verkle_trie::proof::UpdateHint;
use ark_serialize::{CanonicalSerialize};

use std::io::Write;

type Nd = usize;
type Ed<'a> = &'a (usize, usize);
struct Graph { nodes: Vec<&'static str>, edges: Vec<(usize,usize)> }

pub fn render_to<W: Write>(output: &mut W) {
    let nodes = vec!("{x,y}","{x}","{y}","{}");
    let edges = vec!((0,1), (0,2), (1,3), (2,3));
    let graph = Graph { nodes: nodes, edges: edges };

    dot::render(&graph, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example2").unwrap() }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes[*n].into())
    }
    fn edge_label<'b>(&'b self, _: &Ed) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr("lalala".into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
    fn nodes(&self) -> dot::Nodes<'a,Nd> { (0..self.nodes.len()).collect() }
    fn edges(&'a self) -> dot::Edges<'a,Ed<'a>> { self.edges.iter().collect() }
    fn source(&self, e: &Ed) -> Nd { e.0 }
    fn target(&self, e: &Ed) -> Nd { e.1 }
}

pub fn to_dot(uh: &UpdateHint) {
    use std::fs::File;
    let mut f = File::create("example3.dot").unwrap();
    render_to(&mut f);

    for (path, comm) in uh.commitments_by_path.clone() {
        println!("\tnode");
        let mut v = Vec::new();
        comm.serialize(&mut v).unwrap();
        println!("comm :\t{:?}", hex::encode(v));
        print!("path :\t");
        for el in path {
            print!("{:x?} ", el);
        }
        println!();
    }
    println!("extra-info1:\n\n");

    for (val1, val2) in uh.depths_and_ext_by_stem.clone() {
        println!("val1 : {}", hex::encode(val1));
        println!("val2 : {:?}", val2);
    }
    // println!("extra-info2:\n\n");
    // println!("{:?}", val.other_stems_by_prefix);
}