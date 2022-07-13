# blockscout-verkle-tree
Module for visualizing Verkle tree proofs
## Theory
Verkle trie is quite similar to **Modified Merkle Patricia Trie**. To understand how this data structure works, let's look at each modification separately.
#### Merkle Tree

![Hash_Tree](https://user-images.githubusercontent.com/70902141/176688119-ed80ef9e-1c73-4a41-bb61-fb44f5ac7622.png)

The binary tree that is used like **hash function** for any split data.  
Recursively taking the hash function and concatenating, we get the Top Hash, which serves as a digital signature of the data.  

![Hash_Tree_proof](https://user-images.githubusercontent.com/70902141/176688968-0b3e06f6-1a53-4f94-a7c9-2e22da48291d.png)


The *Merkle Proof* requires storing a path for each leaf. This proof is used in Bitcoin blockchain (`merkle_root` in the header).
But there is the main *Merkle tree traversal problem*: it is too expensive to store all the auth-paths in memory.

[Wiki_ru](https://ru.wikipedia.org/wiki/Дерево_хешей). Algorithm complexity: *O(ln(N))* (proof).
### Trie
This is k-ary search tree. Also called *prefix-tree* for strings.  

![539-5390088_trie-example-hd-png-download](https://user-images.githubusercontent.com/70902141/176702178-f4668836-f14c-4bd5-a809-5ddcf14ffd7b.png)  
*Example for set of words: "A", "to", "tea", "ted", "ten", "i", "in", and "inn".inside *  

Sample of *Node* struct:
```
template <class T>
class className {
    Node children[ALPHABET_SIZE];
    bool is_terminal;
    T value;
};
```
Algorithm complexity: *O(N<sup>2</sup>)* : create; *O(m)* : find, where m - lenght of word;

#### Patricia trie
  
![An_example_of_how_to_find_a_string_in_a_Patricia_trie](https://user-images.githubusercontent.com/70902141/176709476-35e62471-0b8a-43c0-923a-120e856417c9.png)  
This is a prefix tree in which prefixes are binary — that is, each key node stores information about one bit. In Ethereum the transition takes place according to the heximal number system (wich called nibble).

### Modified Merkle Patricia Trie
Here it is. The main features that distinguish Ethereum from Bitcoin are implemented by realization of ***this data structure***.  
Modified Merkle Patricia Trie (MPT) provides opportunities to store account statuses, smart-contracts and almost anything dynamically stored data ***inside the blockchain***.  
There are a lot of optimizations which currently Ethereum use, I'll keep it out in context of this README.  
You can find Ethereum docs about MPT [here](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie).
![YZGxe](https://user-images.githubusercontent.com/70902141/177598321-aa02c6bf-93e6-488e-aadb-0cd8826e3ded.png)  
In short, we combine everything that we talked about above.  
We dynamically store the key-value, and verify their authenticity with a digital signature at the root (as Merkle Tree).  
Asymptotics of methods like in prefix tree. There are a few difficult parts in realization, but not critical ([article with realization](https://habr.com/ru/post/446558/)).

### Verkle Tree  
I find [Vitalik 's article](https://vitalik.ca/general/2021/06/18/verkle.html) great for beginning.  
Verkle Trees are very similar to *Modified Merkle Patricia Trie*. The main difference is complexity of cryptographic part.  
  
There are the same constuction of tree: a leaf nodes with key-value, intermediate nodes with `width` number of children (in MPT is constantly 16). The main idea of creating this data structure is ***reduce the size of all data***. Let's take a look at some pictures:  
  
![image](https://user-images.githubusercontent.com/70902141/178154031-93df575b-9528-4666-ae66-a461ed80662d.png)  

Nothing new for us yet, but let's see what will *Merkle tree* require for a single proof.  
  
![image](https://user-images.githubusercontent.com/70902141/178154220-cff1b132-385e-4be6-858b-0e836917f4b7.png)  
  
That's a lot of nodes! Meanwhile *Verkle Tree* require only 3 (!) nodes.

![image](https://user-images.githubusercontent.com/70902141/178154334-a9a0dc3b-eecc-4aea-9eb7-14abc6512063.png)
  
That's impressive! So, how it works?

*Merkle tree* and *Merkle proof* is algorithm that we can relate to **vector commitment**. And not to go into the depths of group theory, we can assume this is one of the easiest one. In turn, *Verkle Tree* use **polynomial commitment**.
>Polynomial commitments let you hash a polynomial, and make a proof for the evaluation of the hashed polynomial at *any* point.  

There are two the easiest polynomial commitment schemes: [KZG commitmens](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html), [ bulletproof-style commitments](https://twitter.com/VitalikButerin/status/1371844878968176647).  

Using these schemes already have a big win, but the math we use here provide us opportunity to **merge** different proofs (like on the last picture).

**Time Complexities** (from [this](https://math.mit.edu/research/highschool/primes/materials/2018/Kuszmaul.pdf) paper)  

![image](https://user-images.githubusercontent.com/70902141/178260990-86724a74-61d1-4c49-9e0d-817b208ed08f.png)
  
k here - is `width`. Because *Verkle Tree* doesn't have multiplier `width - 1` while proof (we don't have to get sisters), **proof** will be much better.
  
#### Usefull links
  
I consider there's no point in implementation of *verkle tree* (i found it).  
So, if someone comes back here in the future, I'll leave **a bunch** of links here, which helped me a lot.

| links | description |
| --- | --- |
| [crate-crypto/rust-verkle](https://github.com/crate-crypto/rust-verkle) | **Implementation** of verkle tree in *rust* (special crate) |
| [gballet/verkle-block-sample](https://github.com/gballet/verkle-block-sample) | **Example** of verklee proof in rust |
| [condrieu](https://condrieu.ethdevops.io/) | **Testnet** with usage of verkle tree |
| [gballet/go-verkle](https://github.com/gballet/go-verkle) | **Test client** of verkle tree in go |
| [Ethereum Meeting](https://www.youtube.com/watch?v=1hTscLYsaIg&t=1167s&ab_channel=EthereumCatHerders) | recording a meeting, where Vitalik talks about verkle tree (beginning) |
| [Guillaume Ballet on *ETH PRAGUE 2020*](https://www.youtube.com/watch?v=4fL7hi8SZMs&ab_channel=ParallelPolis) | Speech by Guillaume Ballet about transition to verkle-networks |
| [eth research](https://ethresear.ch/t/a-minimum-viable-kzg-polynomial-commitment-scheme-implementation/7675) | **KZG commitments** research |
| [MIT paper](https://math.mit.edu/research/highschool/primes/materials/2018/Kuszmaul.pdf) | **Intro-paper** to verkle trees (beginning) |
| [Vitalik's paper](https://vitalik.ca/general/2021/06/18/verkle.html) | **Post** a bit harder than previous (beginning) |
| [verkle-trie-for-eth1](https://dankradfeist.de/ethereum/2021/06/18/verkle-trie-for-eth1.html) | **Post** to read (beginning) |
| [KZG commitments](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html) | **Post** about Kate commitments (some math) |
| [Math](https://vitalik.ca/general/2017/01/14/exploring_ecp.html) | Elliptic Curve usage |

