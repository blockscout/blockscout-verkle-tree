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
