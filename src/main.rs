mod sharky_memory;
mod sharky_vm;

fn main() {
    let mut stack = sharky_memory::SharkyFrame::default();
    stack.push(sharky_memory::SharkyDataType::Max(1_000_000));
    stack.push(sharky_memory::SharkyDataType::Max(2_000_000));
    stack.push(sharky_memory::SharkyDataType::Bool(false));
    let val = stack.get(2);
    let size = stack.size();
    println!("Top value: {val}, Length: {size}");
}
