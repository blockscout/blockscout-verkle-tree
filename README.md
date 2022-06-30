# blockscout-verkle-tree
Module for visualizing Verkle tree proofs
## Theory
Verkle trie is quite similar to **Modified Merkle Patricia Trie**. To understand how this data structure works, let's look at each modification separately.
### Merkle Tree
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
*Example for set of words: "A", "to", "tea", "ted", "ten", "i", "in", and "inn".*  

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

### Patricia trie
![An_example_of_how_to_find_a_string_in_a_Patricia_trie](https://user-images.githubusercontent.com/70902141/176709476-35e62471-0b8a-43c0-923a-120e856417c9.png)  
This is a prefix tree in which prefixes are binary — that is, each key node stores information about one bit. In Ethereum the transition takes place according to the heximal number system (wich called nibble).

### Modified Merkle Patricia Trie


