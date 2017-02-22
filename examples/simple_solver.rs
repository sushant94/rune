extern crate rune;
extern crate r2pipe;

use rune::explorer::explorer::PathExplorer;
use rune::explorer::directed_explorer::DirectedExplorer;
use rune::context::ssa_ctx;
use rune::engine::engine::Engine;
use rune::engine::rune::Rune;
use rune::context::context::ContextAPI;
use r2pipe::r2::R2;
use std::collections::HashMap;

fn main() {
    let mut stream = R2::new(Some("./test_files/newcrackme")).expect("Unable to spawn r2");
    stream.init();

    let mut var_map: HashMap<String, u64> = HashMap::new();

    var_map.insert("rbp".to_owned(), 0x9000);
    var_map.insert("rsp".to_owned(), 512);
    var_map.insert("of".to_owned(), 0);
    var_map.insert("cf".to_owned(), 0);
    var_map.insert("zf".to_owned(), 0);
    var_map.insert("pf".to_owned(), 0);
    var_map.insert("sf".to_owned(), 0);
    var_map.insert("rax".to_owned(), 0);
    var_map.insert("rdx".to_owned(), 0);
    var_map.insert("rsi".to_owned(), 0);
    var_map.insert("rdi".to_owned(), 0);

    let mut ctx = ssa_ctx::new_ssa_ctx(Some(0x0040060a), Some(Vec::new()), Some(var_map.clone()));
    let mut explorer = DirectedExplorer::new();
    
    let mut v: Vec<(u64, char)> = Vec::new();
    v.push((0x0040061d, 'F'));
    v.push((0x0040062b, 'F'));
    v.push((0x00400632, 'F'));
    v.push((0x00400643, 'F'));

    explorer.break_addr = 0x00400643;

    explorer.set_decisions(v);

    let mut sym_mem_vec = Vec::new();
    sym_mem_vec.push(0x8fe0);
    sym_mem_vec.push(0x8fe8);
    sym_mem_vec.push(0x8ff8);
    sym_mem_vec.push(0x8ff0);

    for addr in sym_mem_vec {
        ctx.set_mem_as_sym(addr as usize, 64);
    }

    let mut rune = Rune::new(ctx, explorer, stream);
    rune.run().expect("not yet implemented");
}
