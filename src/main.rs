mod sharky_memory;

fn main() {
    let mut stack = sharky_memory::SharkyFrame::default();
    stack.push(sharky_memory::SharkyDataType::Max(1_000_000));
    stack.push(sharky_memory::SharkyDataType::Max(2_000_000));
    stack.push(sharky_memory::SharkyDataType::Bool(true));
    let val = stack.get(2);
    println!("Top value: {val}");
}
