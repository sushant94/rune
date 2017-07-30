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
use libsmt::theories::{integer, array_ex, bitvec, core};
use r2api::structs::Endian;

use memory::memory::Memory;

#[derive(Copy, Clone, Debug)]
pub struct MemRange {
    start: u64,
    end: u64
}

impl MemRange {
    fn new(start: u64, end: u64) -> MemRange {
        MemRange {
            start: start,
            end: end,
        }
    }

    fn get_width(&self) -> usize {
        (self.end - self.start) as usize
    }

    fn get_overlap(&self, other: MemRange) -> Option<RangeCondition> {
        if self.start < other.start && self.end > other.start && self.end < other.end {
            Some(RangeCondition::Contained(other, other.start, self.end))
        } else if self.start > other.start && self.end < other.end {
            Some(RangeCondition::Full(other))
        } else if self.start < other.end && self.end > other.end {
            Some(RangeCondition::Contained(other, self.start, other.start))
        } else {
        }
    }
}

impl PartialEq for MemRange {
    fn eq(&self, other:&MemRange) -> bool {
        if self.start == other.start && self.end == other.end {
            true
        } else {
            false
        }
    }
}

impl Eq for MemRange {}

impl PartialOrd for MemRange {
    fn partial_cmp(&self, other: &MemRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemRange {
    fn cmp(&self, other: &MemRange) -> Ordering {
        if self.start == other.start && self.end == other.end {
            Ordering::Equal
        } else if self.start < other.start {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[derive(Clone, Debug)]
pub struct MemBlock {
    range: MemRange,
    solver_idx: Option<NodeIndex>,
}

impl MemBlock {
    pub fn new(m_range: MemRange, solver_idx: Option<NodeIndex>) -> MemBlock {
        MemBlock {
            range: m_range,
            solver_idx: solver_idx,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SegMem {
    addr_width: usize,
    endian: Endian,
    segments: BTreeMap<MemRange, MemBlock>,
}
// A function which ->
// Given a vec of ranges and a range gives:
// vec of results which specify different parts of the given range lie in different segments
#[derive(Clone, Debug, Copy)]
pub enum RangeCondition {
    Full(MemRange),
    Contained(MemRange, u64, u64),
    Free(u64, u64),
}

impl SegMem {
    fn get_range_conditions(&self, read_range: MemRange) -> Option<Vec<RangeCondition>> {
        let mem = self.segments.clone();
        let ranges: Vec<MemRange> = mem.keys().cloned().collect();

        let mut pos = match ranges.binary_search(&&read_range) {
            Ok(0) | Err(0) => 0,
            Ok(pos) | Err(pos) => pos - 1,
        };

        let mut conditions = Vec::new();

        let mut overlap = Some(RangeCondition::Free(0, 0));
        let mut other: MemRange;

        println!("{:?}", ranges);

        if ranges.is_empty() {
            conditions.push(RangeCondition::Free(read_range.start, read_range.end));
        } else { 
            while overlap.is_some() {
                other = ranges[pos];
                overlap = read_range.get_overlap(other);
                if overlap.is_some() {
                    conditions.push(overlap.unwrap());
                }
                pos += 1;
                println!("{:?}", overlap);
            }
        }

        Some(conditions)
    }
}

impl Memory for SegMem {
    type VarRef = NodeIndex;

    fn new(address_width: usize, endian: Endian) -> SegMem {
        SegMem {
            addr_width: address_width, 
            endian: endian,
            segments: BTreeMap::new(),
        }
    }

    fn init_memory(&mut self, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        self.segments = BTreeMap::new();
    }

    fn read(&mut self, addr: NodeIndex, read_size: usize, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        // Assert that address is valid
        let addr = match solver.get_node_info(addr) {
            &BVOps(Const(x, _)) => x,
            _ => panic!("Reading from invalid addr!")
        };

        let mut segments = self.segments.clone();

        let read_range = MemRange::new(addr, addr+read_size as u64);
        let conditions = self.get_range_conditions(read_range).unwrap();

        println!("{:?}", conditions);

        let mut result = solver.new_const(bitvec::OpCodes::Const(0, read_range.get_width()));

        for cond in &conditions {
            match *cond {
                RangeCondition::Free(x, y) => {
                    let key = format!("mem_{}_{}", x, y-x);
                    let new_var = solver.new_var(Some(&key), qf_abv::bv_sort((y-x) as usize));

                    let m_range = MemRange::new(x, y);
                    segments.insert(m_range, MemBlock::new(m_range, Some(new_var)));
                },
                RangeCondition::Full(x) => {
                },
                RangeCondition::Contained(r, x, y) => {
                },
                RangeCondition::Contained(r, x, y) => {
                },
            }
        }

        self.segments = segments;
        result
        /*

        let read_range = MemRange { start: addr, end: addr + read_size as u64 };
        let mut mem_block = MemBlock { range: read_range, solver_idx: None };

        match mem.binary_search(&mem_block) {
            Ok(pos) => {
                // Assume it lies completely inside
                let current_block = mem.get(pos);
                let low = read_range.start - current_block.range.start;
                let high = read_range.end - current_block.range.start;

                solver.assert(bitvec::OpCodes::Extract(high - 1, low), &[current_block.solver_idx])
            },
            Err(pos) => {
                let bv_array = qf_abv::bv_sort(read_size);
                let node = solver.new_var(Some(format!("mem_{}_{}", addr, read_size)), bv_array);
                mem_block.solver_idx = Some(node));
                mem.insert(mem_block);

                node
            }
        }
        */
    }

    /*
     * Algorithm
     * Check if node already present
     * If not, create -> Create intermediary nodes
     */
    fn write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: usize, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        /*
        // Check if memory is initialized
        if self.roots.is_none() {
            self.init_memory();
        }

        // Assert that address is valid
        let addr = match solver.get_node_info(addr) {
            &BVOps(Const(x, _)) => x,
            _ => panic!("Reading from invalid addr")
        };

        let mut mem = self.roots.unwrap();

        let write_range = MemRange { start: addr, end: addr + write_size as u64 };
        let mut mem_block = MemBlock { range: write_range, solver_idx: None };

        match mem.binary_search(&mem_block) {
            Ok(pos) => {

            },
            Err(pos) => {
                panic!(
            }
        }
        */
    }
}

mod test {
    use super::*;

    #[test]
    fn check_read() {
        let mut solver = SMTLib2::new(Some(qf_abv::QF_ABV));
        let addr = solver.new_const(bitvec::OpCodes::Const(100, 64));
        let mut mem = SegMem::new(64, Endian::Big);
        let block = mem.read(addr, 100, &mut solver);
        println!("{}", solver.generate_asserts());
        println!("------------------");
        let addr2 = solver.new_const(bitvec::OpCodes::Const(200, 64));
        let block2 = mem.read(addr2, 100, &mut solver);
        println!("{}", solver.generate_asserts());
        println!("------------------");
        let addr3 = solver.new_const(bitvec::OpCodes::Const(150, 64));
        let block3 = mem.read(addr3, 100, &mut solver);
        println!("{}", solver.generate_asserts());
        println!("------------------");
        /*
        let data = solver.new_const(bitvec::OpCodes::Const(24, 32));
        mem.write(addr, data, 32, &mut solver);
        println!("{}", solver.generate_asserts());
        */
    }
}
