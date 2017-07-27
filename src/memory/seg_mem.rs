use petgraph::graph::NodeIndex;
use petgraph::graph::Graph;
use petgraph::Direction;

use std::collections::BTreeMap;
use std::cmp::Ordering;

use libsmt::backends::smtlib2::{SMTLib2, SMTProc};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::logics::qf_abv::QF_ABV_Fn::BVOps;
use libsmt::theories::bitvec::OpCodes::Const;
use libsmt::theories::{array_ex, bitvec, core};
use r2api::structs::Endian;

use memory::memory::Memory;

#[derive(Copy, Clone, Debug)]
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

impl PartialEq for MemRange {
    fn eq(&self, other: &MemRange) -> bool {
        other.start >= self.start && other.start < self.end
   }
}

impl Eq for MemRange {}

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
pub struct SegMem {
    addr_width: usize,
    endian: Endian,
    roots: BTreeMap<MemRange, NodeIndex>,
    g: Graph<MemBlock, Direction>,
}

/*
 * Overall idea:
 *           MEM_BLOCK
 *      /               \
 *  MEM_BLOCK(0.5)    MEM_BLOCK(0.5)
 */
impl Memory for SegMem {
    type VarRef = NodeIndex;

    fn new(address_width: usize, endian: Endian) -> SegMem {
        SegMem {
            addr_width: address_width, 
            endian: endian,
            roots: BTreeMap::new(),
            g: Graph::new(),
        }
    }

    // Kinda useless but okay.
    fn init_memory(&mut self, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        self.roots = BTreeMap::new();
        self.g = Graph::new();
    }

   /*
    * Algorithm
    * 1. Check if node already present
    * 2. If not, create and return
    * */
    fn read(&mut self, addr: NodeIndex, read_size: usize, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        let addr = match solver.get_node_info(addr) {
            &BVOps(Const(x, _)) => x,
            _ => panic!("Reading from invalid addr!")
        };
        let read_range = MemRange { start: addr, end: (addr + read_size as u64)};
        if let Some(_) = self.roots.get(&read_range) {
            // Start of the read lies in some block we already have.
            panic!("Unimplemented!")
        } else {
            let bv_array = qf_abv::bv_sort(read_size);
            let node = solver.new_var(Some(format!("mem_{}_{}", addr, read_size)), bv_array);
            let m_range = MemRange { start: addr, end: addr + read_size as u64 };
            let m_block = MemBlock { range: m_range, solver_idx: node };
            let root = self.g.add_node(m_block);
            self.roots.insert(m_range, root);
            node
        }
    }

    /*
     * Algorithm
     * Check if node already present
     * If not, create -> Create intermediary nodes
     */
    fn write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: usize, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        /*
        if addr in roots(check-ranges):
            if writesize is >:
                create higher nodes:
            if writesize is <:
                create intermediary nodes
            if writesize is same:
                write
        else:
            create_node
            */
    }
}

mod test {
    use super::*;

    #[test]
    fn check_read() {
        let mut solver = SMTLib2::new(Some(qf_abv::QF_ABV));
        let addr = solver.new_const(bitvec::OpCodes::Const(0x9000, 64));
        let mut mem = SegMem::new(64, Endian::Big);
        mem.read(addr, 64, &mut solver);
    }
}
