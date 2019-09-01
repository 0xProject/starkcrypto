use crate::{hash::Hash, hashable::Hashable, masked_keccak::MaskedKeccak};
use std::prelude::v1::*;

#[cfg(feature = "prover")]
use rayon::prelude::*;
#[cfg(feature = "prover")]
use std::marker::Sync;

// This trait is for objects where the object is grouped into hashable sets
// based on index before getting made into a merkle tree, with domain size
// being the max index [ie the one which if you iterate up to it splits the
// whole range]
pub trait Groupable<LeafType: Hashable> {
    fn get_leaf_hash(&self, index: usize) -> Hash {
        self.get_leaf(index).hash()
    }
    fn get_leaf(&self, index: usize) -> LeafType;
    fn domain_size(&self) -> usize;
}

// This trait is applied to give groupable objects a merkle tree based on their
// groupings
pub trait Merkleizable<NodeHash: Hashable> {
    fn merkleize(self) -> Vec<Hash>;
}

struct MerkleNode<'a>(&'a Hash, &'a Hash);

impl Hashable for MerkleNode<'_> {
    fn hash(&self) -> Hash {
        let mut hasher = MaskedKeccak::new();
        hasher.update(self.0.as_bytes());
        hasher.update(self.1.as_bytes());
        hasher.hash()
    }
}

#[cfg(feature = "prover")]
pub fn make_tree_direct<T: Hashable>(leaves: &[T]) -> Vec<Hash> {
    let n = leaves.len();
    let depth = n.trailing_zeros(); // Log_2 of n
    let layer1_index = 2_usize.pow(depth - 1);
    let mut tree = vec![Hash::default(); n]; // Get my vector heap for end results

    for (index, pair) in leaves.chunks(2).enumerate() {
        tree[layer1_index + index] =
            MerkleNode(&pair[0_usize].hash(), &pair[1_usize].hash()).hash();
    }
    for i in (0..(2_usize.pow(depth - 1))).rev() {
        tree[i] = MerkleNode(&tree[2 * i], &tree[2 * i + 1]).hash();
    }
    tree
}

#[cfg(feature = "prover")]
pub fn make_tree<T: Hashable + Sync>(leaves: &[T]) -> Vec<Hash> {
    if leaves.len() < 256 {
        make_tree_direct(leaves)
    } else {
        make_tree_threaded(leaves)
    }
}

#[cfg(feature = "prover")]
pub fn make_tree_threaded<T: Hashable + Sync>(leaves: &[T]) -> Vec<Hash> {
    let n = leaves.len();
    debug_assert!(n.is_power_of_two());
    let depth = n.trailing_zeros() as usize;

    let mut layers = Vec::with_capacity(depth);
    let mut hold = Vec::with_capacity(n / 2);
    leaves
        .into_par_iter()
        .chunks(2)
        .map(|pair| MerkleNode(&pair[0_usize].hash(), &pair[1_usize].hash()).hash())
        .collect_into_vec(&mut hold);
    layers.push(hold);

    for i in 1..(depth) {
        let mut hold = Vec::with_capacity(layers[i - 1].len() / 2);
        layers[i - 1]
            .clone()
            .into_par_iter()
            .chunks(2)
            .map(|pair| MerkleNode(&pair[0_usize], &pair[1_usize]).hash())
            .collect_into_vec(&mut hold);
        layers.push(hold);
    }
    layers.push(vec![Hash::default()]);

    layers.into_iter().rev().flatten().collect()
}

// Note - Make sure to remove duplicated indexes from the input values.
#[cfg(feature = "prover")]
pub fn proof<R: Hashable, T: Groupable<R>>(
    tree: &[Hash],
    indices: &[usize],
    source: &T,
) -> Vec<Hash> {
    debug_assert!(tree.len().is_power_of_two());
    let depth = tree.len().trailing_zeros();
    let num_leaves = 2_usize.pow(depth);
    let num_nodes = 2 * num_leaves;
    let mut known = vec![false; num_nodes + 1];
    let mut decommitment: Vec<Hash> = Vec::new();

    let mut peekable_indicies = indices.iter().peekable();
    let mut excluded_pair = false;
    for &index in indices.iter() {
        let _ = peekable_indicies.next();
        known[num_leaves + index % num_leaves] = true;

        if index % 2 == 0 {
            known[num_leaves + 1 + index % num_leaves] = true;
            if let Some(x) = peekable_indicies.peek() {
                if **x == index + 1 {
                    excluded_pair = true;
                } else {
                    decommitment.push(source.get_leaf_hash(index + 1));
                }
            } else {
                decommitment.push(source.get_leaf_hash(index + 1));
            }
        } else if excluded_pair {
            known[num_leaves - 1 + index % num_leaves] = true;
            excluded_pair = false;
        } else {
            known[num_leaves - 1 + index % num_leaves] = true;
            decommitment.push(source.get_leaf_hash(index - 1));
        }
    }

    for i in (2_usize.pow(depth - 1))..(2_usize.pow(depth)) {
        let left = known[2 * i];
        let right = known[2 * i + 1];
        known[i] = left || right;
    }

    for d in (1..depth).rev() {
        for i in (2_usize.pow(d - 1))..(2_usize.pow(d)) {
            let left = known[2 * i];
            let right = known[2 * i + 1];
            if left && !right {
                decommitment.push(tree[2 * i + 1].clone());
            }
            if !left && right {
                decommitment.push(tree[2 * i].clone());
            }
            known[i] = left || right;
        }
    }
    decommitment
}

pub fn decommitment_size(indices: &[usize], data_size: usize) -> usize {
    let depth = data_size.trailing_zeros();
    let num_leaves = 2_usize.pow(depth);
    let num_nodes = 2 * num_leaves;
    let mut known = vec![false; num_nodes + 1];
    let mut total = 0;

    let mut peekable_indicies = indices.iter().peekable();
    let mut excluded_pair = false;
    for &index in indices.iter() {
        // TODO: Use return value.
        let _ = peekable_indicies.next();
        known[num_leaves + index % num_leaves] = true;

        if index % 2 == 0 {
            known[num_leaves + 1 + index % num_leaves] = true;
            if let Some(x) = peekable_indicies.peek() {
                if **x == index + 1 {
                    excluded_pair = true;
                } else {
                    total += 1;
                }
            } else {
                total += 1;
            }
        } else if excluded_pair {
            known[num_leaves - 1 + index % num_leaves] = true;
            excluded_pair = false;
        } else {
            known[num_leaves - 1 + index % num_leaves] = true;
            total += 1;
        }
    }

    for i in (2_usize.pow(depth - 1))..(2_usize.pow(depth)) {
        let left = known[2 * i];
        let right = known[2 * i + 1];
        known[i] = left || right;
    }

    for d in (1..depth).rev() {
        for i in (2_usize.pow(d - 1))..(2_usize.pow(d)) {
            let left = known[2 * i];
            let right = known[2 * i + 1];
            if left && !right {
                total += 1;
            }
            if !left && right {
                total += 1;
            }
            known[i] = left || right;
        }
    }
    total
}

pub fn verify<T: Hashable>(
    root: &Hash,
    depth: u32,
    values: &mut [(usize, T)],
    decommitment: &[Hash],
) -> bool {
    let mut queue = Vec::with_capacity(values.len());
    let mut previous_index = 0;
    for leaf in values.iter().rev() {
        if leaf.0 % 2 == 1 || previous_index != leaf.0 + 1 {
            let tree_index = 2_usize.pow(depth) + leaf.0;
            queue.push((tree_index, leaf.1.hash()));
            previous_index = leaf.0;
        } else if !(decommitment.iter().any(|x| *x == leaf.1.hash())) {
            let tree_index = 2_usize.pow(depth) + leaf.0;
            queue.push((tree_index, leaf.1.hash()));
        }
    }

    let mut consumed = 0;
    let mut decommitment_iter = decommitment[0..0].iter().rev();
    loop {
        if queue.len() == 1 && queue[0].0 == 1 {
            debug_assert_eq!(decommitment.len(), consumed);
            return queue[0].1 == *root;
        }

        let mut new_queue = Vec::new();
        let pairs = count_pairs(queue.as_slice());

        if consumed < decommitment.len() {
            decommitment_iter = decommitment[consumed..(consumed + queue.len() - 2 * pairs.len())]
                .iter()
                .rev();
            consumed += queue.len() - 2 * pairs.len();
        }

        let mut index = 0;
        let mut pair_index = 0;
        while index < queue.len() {
            if pairs.len() > pair_index && index == pairs[pair_index] {
                new_queue.push((
                    queue[index].0 / 2,
                    MerkleNode(&queue[index + 1].1, &queue[index].1).hash(),
                ));
                index += 2;
                pair_index += 1;
            } else {
                if queue[index].0 % 2 == 0 {
                    let other_hash = decommitment_iter.next().expect("Bad decommitment");
                    new_queue.push((
                        queue[index].0 / 2,
                        MerkleNode(&queue[index].1, other_hash).hash(),
                    ))
                } else {
                    let other_hash = decommitment_iter.next().expect("Bad decommitment");
                    new_queue.push((
                        queue[index].0 / 2,
                        MerkleNode(other_hash, &queue[index].1).hash(),
                    ));
                }
                index += 1;
            }
        }
        debug_assert_eq!(decommitment_iter.next(), None);
        queue = new_queue;
    }
}

fn count_pairs<T>(domain: &[(usize, T)]) -> Vec<usize> {
    let mut previous = &domain[0];
    let mut pairs = Vec::new();
    for (index, item) in domain[1..].iter().enumerate() {
        if previous.0 % 2 == 1 && previous.0 - 1 == item.0 {
            pairs.push(index);
        }
        previous = item;
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use macros_decl::hex;
    use u256::U256;

    impl Groupable<U256> for &[U256] {
        fn get_leaf(&self, index: usize) -> U256 {
            self[index].clone()
        }

        fn domain_size(&self) -> usize {
            self.len()
        }
    }

    #[test]
    fn test_merkle_creation_and_proof() {
        let depth = 6;
        let mut leaves = Vec::new();

        for i in 0..2_u64.pow(depth) {
            leaves.push(U256::from((i + 10).pow(3)));
        }

        let tree = make_tree(leaves.as_slice());

        assert_eq!(
            tree[1].as_bytes(),
            hex!("fd112f44bc944f33e2567f86eea202350913b11c000000000000000000000000")
        );
        let mut values = vec![
            (1, leaves[1].clone()),
            (10, leaves[10].clone()),
            (11, leaves[11].clone()),
            (14, leaves[14].clone()),
        ];

        let indices = vec![1, 11, 14];
        let decommitment = proof(tree.as_slice(), &indices, &leaves.as_slice());
        let non_root = Hash::new(hex!(
            "ed112f44bc944f33e2567f86eea202350913b11c000000000000000000000000"
        ));

        assert!(verify(
            &tree[1],
            depth,
            values.as_mut_slice(),
            &decommitment
        ));
        assert!(!verify(
            &non_root,
            depth,
            values.as_mut_slice(),
            &decommitment
        ));
    }
}