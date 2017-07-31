use petgraph::graph::NodeIndex;
use petgraph::graph::Graph;
use petgraph::Direction;

use std::collections::BTreeMap;
use std::cmp::Ordering;

use libsmt::backends::smtlib2::{SMTLib2, SMTProc};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::logics::qf_abv::QF_ABV_Fn::BVOps;
use libsmt::theories::bitvec::OpCodes::*;
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

    fn contains(&self, num: u64) -> bool {
        if num >= self.start && num < self.end {
            true
        } else {
            false
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

impl SegMem {
    fn read_segment(&mut self, READ: u64, start: u64, end: u64, low: u64, high: u64, width: u64, e_mem: Option<NodeIndex>, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        let ext = width - (high - low); 
        let shift = width - (READ + (high - low));

        if let Some(mem) = e_mem {
            let int1 = solver.assert(Extract(low, high), &[mem]);
            let int2 = solver.assert(ZeroExtend(ext), &[int1]);
            let int3 = solver.assert(BvShl, &[int1, int2]);

            int3
        } else {
            let size = high - low;
            let key = format!("mem_{}_{}", start, size);
            let int1 = solver.new_var(Some(&key), qf_abv::bv_sort(size as usize));

            let int2 = solver.assert(ZeroExtend(ext), &[int1]);
            let int3 = solver.new_const(Const(shift, width as usize));
            let int4 = solver.assert(BvShl, &[int2, int3]);
            
            let r = MemRange::new(start, end);
            let m = MemBlock::new(r, Some(int1));

            self.segments.insert(r, m);

            int4
        }
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

        let read_range = MemRange::new(addr, addr+read_size as u64);

        let mem = self.segments.clone();
        let mut ranges: Vec<MemRange> = mem.keys().cloned().collect();

        let mut pos = match ranges.binary_search(&&read_range) {
            Ok(0) | Err(0) => 0,
            Ok(pos) | Err(pos) => pos - 1,
        };

        let mut iterator = ranges.split_at(pos).1.iter().peekable();

        let mut low: u64;
        let mut high: u64;

        let width = read_range.get_width() as u64;
        let START = read_range.start;
        let END = read_range.end;

        let mut ptr = START;
        let mut cov = 0;

        println!("{} {}", START, END);

        let mut result = solver.new_const(Const(0, width as usize));

        while ptr != END {
            cov = ptr - START;

            if let Some(&current) = iterator.peek() {
                if current.contains(ptr) && current.contains(END) {
                    // extract entire from current
                    println!("extracting entire from {:?}", current);
                    let node = mem.get(&current).unwrap();
                    let node_idx = node.solver_idx.unwrap();

                    low = ptr - current.start;
                    high = END - current.start;

                    let int = self.read_segment(cov, ptr, END, low, high, width, Some(node_idx), solver);
                    result = solver.assert(BvOr, &[result, int]);

                    ptr = END;
                } else if current.contains(ptr) && !current.contains(END) {
                    // extract till end of current
                    println!("extracting till end of {:?}", current);
                    let node = mem.get(&current).unwrap();
                    let node_idx = node.solver_idx.unwrap();

                    low = ptr - current.start;
                    high = current.end - current.start;

                    let int = self.read_segment(cov, ptr, current.end, low, high, width, Some(node_idx), solver);
                    result = solver.assert(BvOr, &[result, int]);

                    ptr = current.end;
                    iterator.next();
                } else if current.start < END && current.end >= END {
                    // create free var till current.start
                    println!("free var till {}", current.start);
                    low = 0;
                    high = current.start - ptr;
                    
                    let int = self.read_segment(cov, ptr, current.start, low, high, width, None, solver);
                    result = solver.assert(BvOr, &[result, int]);
                    
                    ptr = current.start;
                } else {
                    // create free var till end
                    println!("free var till {}", END);
                    low = 0;
                    high = END - ptr;
                    
                    let int = self.read_segment(cov, ptr, END, low, high, width, None, solver);
                    result = solver.assert(BvOr, &[result, int]);

                    ptr = END;
                }
            } else {
                // create free var till end
                println!("free var till {}", END);
                low = 0;
                high = END - ptr;
                
                let int = self.read_segment(cov, ptr, END, low, high, width, None, solver);
                result = solver.assert(BvOr, &[result, int]);

                ptr = END;
            }
        }

        result
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
        let addr2 = solver.new_const(bitvec::OpCodes::Const(300, 64));
        let block2 = mem.read(addr2, 100, &mut solver);
        println!("{}", solver.generate_asserts());
        println!("------------------");
        let addr3 = solver.new_const(bitvec::OpCodes::Const(150, 64));
        let block3 = mem.read(addr3, 200, &mut solver);
        println!("{}", solver.generate_asserts());
        println!("------------------");
        panic!("ZZ")
        /*
        let data = solver.new_const(bitvec::OpCodes::Const(24, 32));
        mem.write(addr, data, 32, &mut solver);
        println!("{}", solver.generate_asserts());
        */
    }
}
