//! Provides serialization of IntSet's to a highly compact bitset format as defined in the
//! IFT specification:
//!
//! <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use crate::bitset::BitSetBuilder;
use crate::input_bit_stream::InputBitStream;
use crate::output_bit_stream::OutputBitStream;
use crate::BitSet;
use crate::IntSet;

#[derive(Debug)]
pub struct DecodingError;

impl Error for DecodingError {}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The input data stream was too short to be a valid sparse bit set."
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum BranchFactor {
    Two,
    Four,
    Eight,
    ThirtyTwo,
}

impl IntSet<u32> {
    /// Populate this set with the values obtained from decoding the provided sparse bit set bytes.
    ///
    /// Sparse bit sets are a specialized, compact encoding of bit sets defined in the IFT specification:
    /// <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
    pub fn from_sparse_bit_set(data: &[u8]) -> Result<IntSet<u32>, DecodingError> {
        // This is a direct port of the decoding algorithm from:
        // <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
        let Some((branch_factor, height)) = InputBitStream::<0>::decode_header(data) else {
            return Err(DecodingError);
        };

        let result = match branch_factor {
            BranchFactor::Two => Self::decode_sparse_bit_set_nodes::<2>(data, height),
            BranchFactor::Four => Self::decode_sparse_bit_set_nodes::<4>(data, height),
            BranchFactor::Eight => Self::decode_sparse_bit_set_nodes::<8>(data, height),
            BranchFactor::ThirtyTwo => Self::decode_sparse_bit_set_nodes::<32>(data, height),
        };

        result.map(IntSet::<u32>::from_bitset)
    }

    fn decode_sparse_bit_set_nodes<const BF: u8>(
        data: &[u8],
        height: u8,
    ) -> Result<BitSet, DecodingError> {
        let mut out = BitSet::empty();
        if height == 0 {
            return Ok(out);
        }

        let mut builder = BitSetBuilder::start(&mut out);
        let mut bits = InputBitStream::<BF>::from(data);
        // TODO(garretrieger): estimate initial capacity (maximum is a function of the number of nodes in the bit stream).
        let mut queue = VecDeque::<NextNode>::new();
        queue.push_back(NextNode { start: 0, depth: 1 });

        while let Some(next) = queue.pop_front() {
            let mut bits = bits.next().ok_or(DecodingError)?;

            if bits == 0 {
                // all bits were zeroes which is a special command to completely fill in
                // all integers covered by this node.
                let exp = (height as u32) - next.depth + 1;
                // TODO(garretrieger): implement special insert_range on the builder as well.
                builder
                    .set
                    .insert_range(next.start..=next.start + (BF as u32).pow(exp) - 1);
                continue;
            }

            loop {
                let bit_index = bits.trailing_zeros();
                if bit_index == 32 {
                    break;
                }

                if next.depth == height as u32 {
                    // TODO(garretrieger): further optimize by inserting entire nodes at once (as a bit field).
                    builder.insert(next.start + bit_index);
                } else {
                    let exp = height as u32 - next.depth;
                    queue.push_back(NextNode {
                        start: next.start + bit_index * (BF as u32).pow(exp),
                        depth: next.depth + 1,
                    });
                }

                bits &= !(1 << bit_index); // clear the bit that was just read.
            }
        }

        builder.finish();

        Ok(out)
    }

    /// Encode this set as a sparse bit set byte encoding.
    ///
    /// Sparse bit sets are a specialized, compact encoding of bit sets defined in the IFT specification:
    /// <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
    pub fn to_sparse_bit_set(&self) -> Vec<u8> {
        // TODO(garretrieger): use the heuristic approach from the incxfer
        // implementation to guess the optimal size. Building the set 4 times
        // is costly.
        let mut candidates: Vec<Vec<u8>> = vec![];

        let Some(max_value) = self.last() else {
            return OutputBitStream::new(BranchFactor::Two, 0).into_bytes();
        };

        if BranchFactor::Two.tree_height_for(max_value) < OutputBitStream::MAX_HEIGHT {
            candidates.push(to_sparse_bit_set_with_bf::<2>(self));
        }

        if BranchFactor::Four.tree_height_for(max_value) < OutputBitStream::MAX_HEIGHT {
            candidates.push(to_sparse_bit_set_with_bf::<4>(self));
        }

        if BranchFactor::Eight.tree_height_for(max_value) < OutputBitStream::MAX_HEIGHT {
            candidates.push(to_sparse_bit_set_with_bf::<8>(self));
        }

        if BranchFactor::ThirtyTwo.tree_height_for(max_value) < OutputBitStream::MAX_HEIGHT {
            candidates.push(to_sparse_bit_set_with_bf::<32>(self));
        }

        candidates.into_iter().min_by_key(|f| f.len()).unwrap()
    }
}

/// Encode this set as a sparse bit set byte encoding with a specified branch factor.
///
/// Branch factor can be 2, 4, 8 or 32. It's a compile time constant so that optimized decoding implementations
/// can be generated by the compiler.
///
/// Sparse bit sets are a specialized, compact encoding of bit sets defined in the IFT specification:
/// <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
pub fn to_sparse_bit_set_with_bf<const BF: u8>(set: &IntSet<u32>) -> Vec<u8> {
    let branch_factor = BranchFactor::from_val(BF);
    let Some(max_value) = set.last() else {
        return OutputBitStream::new(branch_factor, 0).into_bytes();
    };
    let mut height = branch_factor.tree_height_for(max_value);
    let mut os = OutputBitStream::new(branch_factor, height);
    let mut nodes: Vec<Node> = vec![];

    // We build the nodes that will comprise the bit stream in reverse order
    // from the last value in the last layer up to the first layer. Then
    // when generating the final stream the order is reversed.
    // The reverse order construction is needed since nodes at the lower layer
    // affect the values in the parent layers.
    let mut indices = set.clone();
    let mut filled_indices = IntSet::<u32>::all();
    while height > 0 {
        (indices, filled_indices) =
            create_layer(branch_factor, indices, filled_indices, &mut nodes);
        height -= 1;
    }

    for node in nodes.iter().rev() {
        match node.node_type {
            NodeType::Standard => os.write_node(node.bits),
            NodeType::Filled => os.write_node(0),
            NodeType::Skip => {}
        };
    }

    os.into_bytes()
}

struct CreateLayerState<'a> {
    // This is the set of indices which are to be set in the layer above this one
    upper_indices: IntSet<u32>,
    // Similarily, this is the set of indices in the layer above this one which are fully filled.
    upper_filled_indices: IntSet<u32>,

    current_node: Option<Node>,
    current_node_filled_bits: u32,
    nodes: &'a mut Vec<Node>,
    child_count: usize,
    nodes_init_length: usize,
    branch_factor: BranchFactor,
}

impl<'a> CreateLayerState<'a> {
    fn commit_current_node(&mut self) {
        let Some(mut node) = self.current_node.take() else {
            // noop if there isn't a node to commit.
            return;
        };
        self.upper_indices.insert(node.parent_index);

        if self.current_node_filled_bits == self.branch_factor.u32_mask() {
            // This node is filled and can thus be represented by a node that is '0'.
            // It's index is recorded so that the parent node can also check if they are filled.
            self.upper_filled_indices.insert(node.parent_index);
            node.node_type = NodeType::Filled;

            if self.nodes_init_length >= self.child_count {
                // Since this node is filled, find all nodes which are children and set them to be skipped in
                // the encoding.
                let children_start_index = self.nodes_init_length.saturating_sub(self.child_count);
                let children_end_index = self.nodes_init_length;
                // TODO(garretrieger): this scans all nodes of the previous layer to find those which are children,
                //   but we can likely limit it to just the children of this node with some extra book keeping.
                for child in &mut self.nodes[children_start_index..children_end_index] {
                    if child.parent_index >= node.parent_index * self.branch_factor.value()
                        && child.parent_index < (node.parent_index + 1) * self.branch_factor.value()
                    {
                        child.node_type = NodeType::Skip;
                    }
                }
            }
        }

        self.nodes.push(node);
        self.current_node_filled_bits = 0;
    }
}

/// Compute the nodes for a layer of the sparse bit set.
///
/// Computes the nodes needed for the layer which contains the indices in
/// 'iter'. The new nodes are appeded to 'nodes'. 'iter' must be sorted
/// in ascending order.
///
/// Returns the set of indices for the layer above.
fn create_layer(
    branch_factor: BranchFactor,
    values: IntSet<u32>,
    filled_values: IntSet<u32>,
    nodes: &mut Vec<Node>,
) -> (IntSet<u32>, IntSet<u32>) {
    let mut state = CreateLayerState {
        upper_indices: IntSet::<u32>::empty(),
        upper_filled_indices: IntSet::<u32>::empty(),
        current_node: None,
        current_node_filled_bits: 0,
        child_count: values.len(),
        nodes_init_length: nodes.len(),
        nodes,
        branch_factor,
    };

    // The nodes array is produced in reverse order and then reversed before final output.
    for v in values.iter().rev() {
        let parent_index = v / branch_factor.value();
        let prev_parent_index = state
            .current_node
            .as_ref()
            .map_or(parent_index, |node| node.parent_index);
        if prev_parent_index != parent_index {
            state.commit_current_node();
        }

        let current_node = state.current_node.get_or_insert(Node {
            bits: 0,
            parent_index,
            node_type: NodeType::Standard,
        });

        let mask = 0b1 << (v % branch_factor.value());
        current_node.bits |= mask;
        if filled_values.contains(v) {
            state.current_node_filled_bits |= mask;
        }
    }

    state.commit_current_node();
    (state.upper_indices, state.upper_filled_indices)
}

enum NodeType {
    Standard,
    Filled,
    Skip,
}

struct Node {
    bits: u32,
    parent_index: u32,
    node_type: NodeType,
}

impl BranchFactor {
    pub(crate) fn value(&self) -> u32 {
        match self {
            BranchFactor::Two => 2,
            BranchFactor::Four => 4,
            BranchFactor::Eight => 8,
            BranchFactor::ThirtyTwo => 32,
        }
    }

    fn tree_height_for(&self, max_value: u32) -> u8 {
        // height H, can represent up to (BF^height) - 1
        let mut height: u32 = 0;
        let mut max_value = max_value;
        loop {
            height += 1;
            max_value >>= self.node_size_log2();
            if max_value == 0 {
                break height as u8;
            }
        }
    }

    fn from_val(val: u8) -> BranchFactor {
        match val {
            2 => BranchFactor::Two,
            4 => BranchFactor::Four,
            8 => BranchFactor::Eight,
            32 => BranchFactor::ThirtyTwo,
            // This should never happen as this is only used internally.
            _ => panic!("Invalid branch factor."),
        }
    }

    fn node_size_log2(&self) -> u32 {
        match self {
            BranchFactor::Two => 1,
            BranchFactor::Four => 2,
            BranchFactor::Eight => 3,
            BranchFactor::ThirtyTwo => 5,
        }
    }

    pub(crate) fn byte_mask(&self) -> u32 {
        match self {
            BranchFactor::Two => 0b00000011,
            BranchFactor::Four => 0b00001111,
            BranchFactor::Eight => 0b11111111,
            BranchFactor::ThirtyTwo => 0b11111111,
        }
    }

    fn u32_mask(&self) -> u32 {
        match self {
            BranchFactor::Two => 0b00000000_00000000_00000000_00000011,
            BranchFactor::Four => 0b00000000_00000000_00000000_00001111,
            BranchFactor::Eight => 0b00000000_00000000_00000000_11111111,
            BranchFactor::ThirtyTwo => 0b11111111_11111111_11111111_11111111,
        }
    }
}

struct NextNode {
    start: u32,
    depth: u32,
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod test {
    use super::*;

    #[test]
    fn spec_example_2() {
        // Test of decoding the example 2 given in the specification.
        // See: <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
        let bytes = [
            0b00001110, 0b00100001, 0b00010001, 0b00000001, 0b00000100, 0b00000010, 0b00001000,
        ];

        let set = IntSet::<u32>::from_sparse_bit_set(&bytes).unwrap();
        let expected: IntSet<u32> = [2, 33, 323].iter().copied().collect();
        assert_eq!(set, expected);
    }

    #[test]
    fn spec_example_3() {
        // Test of decoding the example 3 given in the specification.
        // See: <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
        let bytes = [0b00000000];

        let set = IntSet::<u32>::from_sparse_bit_set(&bytes).unwrap();
        let expected: IntSet<u32> = [].iter().copied().collect();
        assert_eq!(set, expected);
    }

    #[test]
    fn spec_example_4() {
        // Test of decoding the example 4 given in the specification.
        // See: <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>
        let bytes = [0b00001101, 0b00000011, 0b00110001];

        let set = IntSet::<u32>::from_sparse_bit_set(&bytes).unwrap();

        let mut expected: IntSet<u32> = IntSet::<u32>::empty();
        expected.insert_range(0..=17);

        assert_eq!(set, expected);
    }

    #[test]
    fn invalid() {
        // Spec example 2 with one byte missing.
        let bytes = [
            0b00001110, 0b00100001, 0b00010001, 0b00000001, 0b00000100, 0b00000010,
        ];
        assert!(IntSet::<u32>::from_sparse_bit_set(&bytes).is_err());
    }

    #[test]
    fn test_tree_height_for() {
        assert_eq!(BranchFactor::Two.tree_height_for(0), 1);
        assert_eq!(BranchFactor::Two.tree_height_for(1), 1);
        assert_eq!(BranchFactor::Two.tree_height_for(2), 2);
        assert_eq!(BranchFactor::Two.tree_height_for(117), 7);

        assert_eq!(BranchFactor::Four.tree_height_for(0), 1);
        assert_eq!(BranchFactor::Four.tree_height_for(3), 1);
        assert_eq!(BranchFactor::Four.tree_height_for(4), 2);
        assert_eq!(BranchFactor::Four.tree_height_for(63), 3);
        assert_eq!(BranchFactor::Four.tree_height_for(64), 4);

        assert_eq!(BranchFactor::Eight.tree_height_for(0), 1);
        assert_eq!(BranchFactor::Eight.tree_height_for(7), 1);
        assert_eq!(BranchFactor::Eight.tree_height_for(8), 2);
        assert_eq!(BranchFactor::Eight.tree_height_for(32767), 5);
        assert_eq!(BranchFactor::Eight.tree_height_for(32768), 6);

        assert_eq!(BranchFactor::ThirtyTwo.tree_height_for(0), 1);
        assert_eq!(BranchFactor::ThirtyTwo.tree_height_for(31), 1);
        assert_eq!(BranchFactor::ThirtyTwo.tree_height_for(32), 2);
        assert_eq!(BranchFactor::ThirtyTwo.tree_height_for(1_048_575), 4);
        assert_eq!(BranchFactor::ThirtyTwo.tree_height_for(1_048_576), 5);
    }

    #[test]
    fn generate_spec_example_2() {
        // Test of reproducing the encoding of example 2 given
        // in the specification. See:
        // <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>

        let actual_bytes = to_sparse_bit_set_with_bf::<8>(&[2, 33, 323].iter().copied().collect());
        let expected_bytes = [
            0b00001110, 0b00100001, 0b00010001, 0b00000001, 0b00000100, 0b00000010, 0b00001000,
        ];

        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn generate_spec_example_3() {
        // Test of reproducing the encoding of example 3 given
        // in the specification. See:
        // <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>

        let actual_bytes = to_sparse_bit_set_with_bf::<2>(&IntSet::<u32>::empty());
        let expected_bytes = [0b00000000];

        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn generate_spec_example_4() {
        // Test of reproducing the encoding of example 3 given
        // in the specification. See:
        // <https://w3c.github.io/IFT/Overview.html#sparse-bit-set-decoding>

        let actual_bytes = to_sparse_bit_set_with_bf::<4>(&(0..=17).collect());
        let expected_bytes = [0b00001101, 0b0000_0011, 0b0011_0001];

        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_one_level() {
        let actual_bytes = to_sparse_bit_set_with_bf::<8>(&[2, 6].iter().copied().collect());
        let expected_bytes = [0b0_00001_10, 0b01000100];
        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_one_level_filled() {
        let actual_bytes = to_sparse_bit_set_with_bf::<8>(&(0..=7).collect());
        let expected_bytes = [0b0_00001_10, 0b00000000];
        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_two_level_filled() {
        let actual_bytes = to_sparse_bit_set_with_bf::<8>(&(3..=21).collect());
        let expected_bytes = [0b0_00010_10, 0b00000111, 0b11111000, 0b00000000, 0b00111111];
        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_two_level_not_filled() {
        let actual_bytes = to_sparse_bit_set_with_bf::<4>(&[0, 4, 8, 12].iter().copied().collect());
        let expected_bytes = [0b0_00010_01, 0b0001_1111, 0b0001_0001, 0b0000_0001];
        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_four_level_filled() {
        let mut s = IntSet::<u32>::empty();
        s.insert_range(64..=127); // Filled node on level 3
        s.insert_range(512..=1023); // Filled node on level 2
        s.insert(4000);

        let actual_bytes = to_sparse_bit_set_with_bf::<8>(&s);
        let expected_bytes = [
            // Header
            0b0_00100_10,
            // L1
            0b10000011,
            // L2
            0b00000010,
            0b00000000,
            0b01000000,
            // L3,
            0b00000000,
            0b00010000,
            // L4
            0b00000001,
        ];
        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn encode_bf32() {
        let actual_bytes = to_sparse_bit_set_with_bf::<32>(&[2, 31, 323].iter().copied().collect());
        let expected_bytes = [
            0b0_00010_11,
            // node 0
            0b00000001,
            0b00000100,
            0b00000000,
            0b00000000,
            // node 1
            0b00000100,
            0b00000000,
            0b00000000,
            0b10000000,
            // node 2
            0b00001000,
            0b00000000,
            0b00000000,
            0b00000000,
        ];

        assert_eq!(actual_bytes, expected_bytes);
    }

    #[test]
    fn round_trip() {
        let s1: IntSet<u32> = [11, 74, 9358].iter().copied().collect();
        let mut s2: IntSet<u32> = s1.clone();
        s2.insert_range(67..=412);

        check_round_trip::<2>(&s1);
        check_round_trip::<4>(&s1);
        check_round_trip::<8>(&s1);
        check_round_trip::<32>(&s1);

        check_round_trip::<2>(&s2);
        check_round_trip::<4>(&s2);
        check_round_trip::<8>(&s2);
        check_round_trip::<32>(&s2);
    }

    fn check_round_trip<const BF: u8>(s: &IntSet<u32>) {
        let bytes = to_sparse_bit_set_with_bf::<BF>(s);
        let s_prime = IntSet::<u32>::from_sparse_bit_set(&bytes).unwrap();
        assert_eq!(*s, s_prime);
    }

    #[test]
    fn find_smallest_bf() {
        let s: IntSet<u32> = [11, 74, 9358].iter().copied().collect();
        let bytes = s.to_sparse_bit_set();
        // BF4
        assert_eq!(vec![0b0_00111_01], bytes[0..1]);

        let s: IntSet<u32> = [
            16, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30,
        ]
        .iter()
        .copied()
        .collect();
        let bytes = s.to_sparse_bit_set();
        // BF32
        assert_eq!(vec![0b0_00001_11], bytes[0..1]);
    }

    #[test]
    fn encode_maxu32() {
        let s: IntSet<u32> = [1, u32::MAX].iter().copied().collect();
        let bytes = s.to_sparse_bit_set();
        let s_prime = IntSet::<u32>::from_sparse_bit_set(&bytes);
        assert_eq!(s, s_prime.unwrap());
    }
}
