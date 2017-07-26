use petgraph::graph::NodeIndex;
use petgraph::graph::Graph;
use petgraph::Direction;

use std::collections::BTreeMap;
use std::cmp::Ordering;

use libsmt::backends::backend::SMTBackend;

#[derive(Clone, Debug)]
struct MemRange {
    start: u64,
    end: u64
}

impl Ord for MemRange {
    fn cmp(&self, other: &MemRange) -> Ordering {
        if other.start >= self.start && other.start < self.end {
            Ordering::Equal
        } else {
            self.start.cmp(&other.start)
        }
    }
}

impl PartialOrd for MemRange {
    fn partial_cmp(&self, other: &MemRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MemRange {
    pub fn contains(&self, element: u64) -> bool {
        if element >= self.start && element < self.end {
            true
        } else {
            false
        }
    }

    pub fn width(&self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemBlock {
    range: MemRange,
    solver_idx: NodeIndex,
}

#[derive(Clone, Debug)]
pub struct RMem {
    addr_width: usize,
    roots: BTreeMap<MemRange, NodeIndex>,
    g: Graph<MemBlock, Direction>,
}

/*
 * Overall idea:
 *           MEM_BLOCK
 *      /               \
 *  MEM_BLOCK(0.5)    MEM_BLOCK(0.5)
 */
impl Memory for RMem {
    type VarRef = NodeIndex;

    fn init_memory<T>(&self, solver: T) where T: SMTBackend {
        self.roots = BTreeMap::new();
        self.g = Graph::new();
    }

   /*
    * Algorithm
    * 1. Check if node already present
    * 2. If not, create and return
    * */
    fn read<T>(&mut self, addr: u64, read_size: usize, solver: T) -> NodeIndex where T: SMTBackend {
        let read_range = MemRange { start: addr, end: (addr + read_size as u64)};
        if let Some(root) = self.roots.get(&read_range) {
            // Compare with existing roots.
        } else {
        }
    }

    fn write<T>(&mut self, addr: u64, data: NodeIndex, write_size: usize, solver: T) where T: SMTBackend {
        /*
         * Algorithm
         * Check if node already present
         * If not, create -> Create intermediary nodes
         */ 
        if addr in roots(check-ranges):
            if writesize is >:
                create higher nodes:
            if writesize is <:
                create intermediary nodes
            if writesize is same:
                write
        else:
            create_node
    }
}
