use verkle_trie::proof::UpdateHint;
use ark_serialize::{CanonicalSerialize};
// use verkle_trie::EdwardsProjective;
use std::borrow::Cow;

use std::io::Write;

type Nd = usize;
type Ed<'a> = &'a (usize, usize, u8);
struct Graph { nodes: Vec<String>, edges: Vec<(usize,usize,u8)> }


pub fn render_to<W: Write>(output: &mut W, data: &UpdateHint) {
    let mut nodes = vec![];
    let mut previous_items = Vec::<(Vec::<u8>, String)>::new();
    let mut edges = vec![];
    let mut my_index = 0;

    for (path, comm) in data.commitments_by_path.clone() {
        let mut v_tmp = vec![];
        let _res = CanonicalSerialize::serialize(&comm, &mut v_tmp);
        let commitment_in_hex: String = format!("0x{}", hex::encode(v_tmp));
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
        previous_items.push((path, commitment_in_hex));
        my_index += 1;
    }

    for (comm, val2) in data.depths_and_ext_by_stem.clone() {
        let commitment_in_hex = format!("0x{}", hex::encode(comm));

        // пройти по префиксному дереву по commitment_in_hex на depth вершин
        // и записать в nodes
        println!("commitment : {}", commitment_in_hex);
        println!("ext and depth : {:?}", val2);
    }

    // пройтись по всем keys и записать их в дерево, если есть ext ноды !
    

    let graph = Graph { nodes: nodes, edges: edges };

    dot::render(&graph, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example2").unwrap() }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes.get(*n).unwrap().into())
    }
    fn edge_label<'b>(&'b self, edge: &Ed) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(Cow::Owned(format!("{:0x}", edge.2)))
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
    render_to(&mut f, &uh);

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
}